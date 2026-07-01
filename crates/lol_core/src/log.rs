use std::path::PathBuf;
use std::sync::OnceLock;

use bevy::log::tracing_subscriber::{Layer, fmt};
use bevy::prelude::*;
use crossbeam_channel::{Sender, unbounded};
use serde::Serialize;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tracing::Event;
use tracing::field::{Field, Visit};

/// 写入端：tracing Layer 把日志发到这个 channel，writer 线程异步落盘。
///
/// 设计说明：tracing 的 `Layer::on_event` 是同步签名（`fn`），而 sqlx 的所有
/// API 都是 async。两者无法直接对接。用 crossbeam 无界 channel 解耦：
/// - Layer 回调只做 `send`（同步、非阻塞），不碰任何 async/DB
/// - writer 线程跑独立的 current_thread tokio runtime，循环 recv → sqlx INSERT
///
/// 这样 `lol_core` 不再依赖 rusqlite，整个 workspace 的 sqlite linker 统一为
/// sqlx-sqlite，避免 `links="sqlite3"` 冲突。
static LOG_TX: OnceLock<Sender<StructuredLog>> = OnceLock::new();

#[derive(Serialize, Clone, Debug)]
pub struct StructuredLog {
    pub timestamp: u64,
    pub level: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    pub message: String,
}

struct StructuredLogVisitor {
    message: String,
    entity_id: Option<u32>,
    entity_name: Option<String>,
    category: Option<String>,
}

impl Visit for StructuredLogVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let name = field.name();
        let val_str = format!("{:?}", value);
        if name == "message" {
            let mut s = val_str;
            if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                s.remove(0);
                s.pop();
            }
            self.message = s;
        } else if name == "entity_id" {
            if let Ok(id) = val_str.parse::<u32>() {
                self.entity_id = Some(id);
            }
        } else if name == "entity_name" {
            self.entity_name = Some(val_str.trim_matches('"').to_string());
        } else if name == "category" {
            self.category = Some(val_str.trim_matches('"').to_string());
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        let name = field.name();
        if name == "message" {
            self.message = value.to_string();
        } else if name == "entity_name" {
            self.entity_name = Some(value.to_string());
        } else if name == "category" {
            self.category = Some(value.to_string());
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() == "entity_id" {
            self.entity_id = Some(value as u32);
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        if field.name() == "entity_id" && value >= 0 {
            self.entity_id = Some(value as u32);
        }
    }
}

pub struct StructuredLogLayer;

impl<S: tracing::Subscriber> Layer<S> for StructuredLogLayer {
    fn on_event(
        &self,
        event: &Event<'_>,
        _ctx: bevy::log::tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let mut visitor = StructuredLogVisitor {
            message: String::new(),
            entity_id: None,
            entity_name: None,
            category: None,
        };
        event.record(&mut visitor);

        if visitor.message.is_empty() {
            visitor.message = metadata.name().to_string();
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let level = metadata.level().to_string().to_lowercase();
        let file = metadata.file().map(|f| f.to_string());
        let line = metadata.line();

        let log = StructuredLog {
            timestamp,
            level,
            file,
            line,
            entity_id: visitor.entity_id,
            entity_name: visitor.entity_name,
            category: visitor.category,
            message: visitor.message,
        };

        // send 到 channel；channel 持有方是 OnceLock，初始化后永远存在。
        // send 失败仅当 writer 线程退出（程序结束），此时丢弃日志是正确的。
        if let Some(tx) = LOG_TX.get() {
            let _ = tx.send(log);
        }
    }
}

struct FilteredLayer<L> {
    inner: L,
    headless: bool,
}

impl<S: tracing::Subscriber, L: Layer<S>> Layer<S> for FilteredLayer<L> {
    fn on_event(
        &self,
        event: &Event<'_>,
        ctx: bevy::log::tracing_subscriber::layer::Context<'_, S>,
    ) {
        if self.headless {
            let mut visitor = StructuredLogVisitor {
                message: String::new(),
                entity_id: None,
                entity_name: None,
                category: None,
            };
            event.record(&mut visitor);
            let message = if visitor.message.is_empty() {
                event.metadata().name()
            } else {
                &visitor.message
            };
            if message.contains("Could not find an asset loader") {
                return;
            }
        }
        self.inner.on_event(event, ctx);
    }
}

/// 初始化 SQLite 日志：建表 + 启动 writer 线程。返回 Bevy LogPlugin。
///
/// writer 线程在自己的 current_thread tokio runtime 上跑，避免和 Bevy 主线程
/// 的 runtime 冲突。channel sender 存入 OnceLock 供 Layer 回调使用。
pub fn create_log_plugin(log_db: Option<PathBuf>) -> bevy::log::LogPlugin {
    let db_path = match log_db {
        Some(p) => p,
        None => {
            let base = std::env::var("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
            base.join(".moon-lol").join("logs").join("debug.db")
        }
    };

    // 确保目录存在
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).expect("无法创建日志目录");
    }

    if LOG_TX.get().is_none() {
        // 每次启动重置日志 DB（沿用原行为）
        if db_path.exists() {
            let _ = std::fs::remove_file(&db_path);
            let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
            let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
        }

        // channel：Layer（生产）→ writer 线程（消费）
        let (tx, rx) = unbounded::<StructuredLog>();
        if LOG_TX.set(tx).is_ok() {
            // writer 线程：独立 current_thread runtime，sqlx async 写
            std::thread::Builder::new()
                .name("lol-core-log-writer".into())
                .spawn(move || {
                    log_writer_main(db_path, rx);
                })
                .expect("无法启动日志 writer 线程");
        }
    }

    let plugin = bevy::log::LogPlugin {
        filter: "bevy_gltf_draco=off".to_owned(),
        fmt_layer: |_app| {
            let format_layer = fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false);

            let combined = format_layer.and_then(StructuredLogLayer);
            let headless = std::env::args().any(|arg| arg == "--headless");
            let filtered = FilteredLayer {
                inner: combined,
                headless,
            };
            Some(Box::new(filtered))
        },
        ..Default::default()
    };

    plugin
}

/// writer 线程主循环：建表 → 循环 recv → INSERT；sender 全部 drop 后排空退出。
fn log_writer_main(db_path: PathBuf, rx: crossbeam_channel::Receiver<StructuredLog>) {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("[lol-core-log] 无法创建 tokio runtime: {e}");
            return;
        }
    };

    rt.block_on(async move {
        let pool = match open_and_init_db(&db_path).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[lol-core-log] 无法初始化 SQLite: {e}");
                return;
            }
        };

        // 循环消费 channel；rx 持有方在 LOG_TX 的 sender 全部 drop 后会返回 Err(Disconnected)，
        // 此时排空剩余日志后退出。
        while let Ok(log) = rx.recv() {
            if let Err(e) = insert_log(&pool, &log).await {
                eprintln!("[lol-core-log] 写入失败: {e}");
            }
        }

        // 优雅关闭：关闭连接池
        pool.close().await;
    });
}

async fn open_and_init_db(db_path: &PathBuf) -> Result<SqlitePool, sqlx::Error> {
    let opts = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            level TEXT NOT NULL,
            file TEXT,
            line INTEGER,
            entity_id INTEGER,
            entity_name TEXT,
            category TEXT,
            message TEXT NOT NULL
        );",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

async fn insert_log(pool: &SqlitePool, log: &StructuredLog) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO logs (timestamp, level, file, line, entity_id, entity_name, category, message)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(log.timestamp as i64)
    .bind(&log.level)
    .bind(&log.file)
    .bind(log.line.map(|l| l as i64))
    .bind(log.entity_id.map(|e| e as i64))
    .bind(&log.entity_name)
    .bind(&log.category)
    .bind(&log.message)
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum EnumLogCategory {
    Run,
    Aggro,
    Turret,
    Skill,
    AttackAuto,
    Attack,
    Movement,
    Minion,
}

#[derive(EntityEvent, Clone, Debug)]
pub struct CommandLog {
    pub entity: Entity,
    pub info: String,
    pub category: EnumLogCategory,
}

#[derive(Default)]
pub struct PluginLog;

impl Plugin for PluginLog {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_log);
    }
}

fn on_command_log(event: On<CommandLog>, q_name: Query<Option<&Name>>) {
    let log_event = event.event();
    let entity = log_event.entity;
    let name = q_name
        .get(entity)
        .ok()
        .flatten()
        .map(|n| n.as_str())
        .unwrap_or("Unknown");

    let category_str = match log_event.category {
        EnumLogCategory::Run => "run",
        EnumLogCategory::Aggro => "aggro",
        EnumLogCategory::Turret => "turret",
        EnumLogCategory::Skill => "skill",
        EnumLogCategory::AttackAuto => "attack_auto",
        EnumLogCategory::Attack => "attack",
        EnumLogCategory::Movement => "movement",
        EnumLogCategory::Minion => "minion",
    };

    debug!(
        category = category_str,
        entity_id = entity.index_u32(),
        entity_name = name,
        "{}",
        log_event.info
    );
}
