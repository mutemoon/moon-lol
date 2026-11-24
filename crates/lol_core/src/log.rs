use bevy::prelude::*;
use serde::Serialize;

#[cfg(target_arch = "wasm32")]
pub fn create_log_plugin(_log_db: Option<std::path::PathBuf>) -> bevy::log::LogPlugin {
    bevy::log::LogPlugin::default()
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::path::PathBuf;
    use std::sync::OnceLock;

    use bevy::log::tracing_subscriber::{Layer, fmt};
    use bevy::prelude::*;
    use crossbeam_channel::{Receiver, Sender, unbounded};
    use serde::Serialize;
    use sqlx::SqlitePool;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use tracing::Event;
    use tracing::field::{Field, Visit};

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
            if name == "message" {
                self.message = format!("{:?}", value);
                if self.message.starts_with('"')
                    && self.message.ends_with('"')
                    && self.message.len() >= 2
                {
                    self.message = self.message[1..self.message.len() - 1].to_string();
                }
            } else if name == "entity_id" {
                if let Ok(id) = format!("{:?}", value).parse::<u32>() {
                    self.entity_id = Some(id);
                }
            } else if name == "entity_name" {
                let s = format!("{:?}", value);
                let trimmed = if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                    &s[1..s.len() - 1]
                } else {
                    &s
                };
                if trimmed != "Unknown" && !trimmed.is_empty() {
                    self.entity_name = Some(trimmed.to_string());
                }
            } else if name == "category" {
                let s = format!("{:?}", value);
                let trimmed = if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                    &s[1..s.len() - 1]
                } else {
                    &s
                };
                self.category = Some(trimmed.to_string());
            }
        }

        fn record_str(&mut self, field: &Field, value: &str) {
            let name = field.name();
            if name == "message" {
                self.message = value.to_string();
            } else if name == "entity_name" {
                if value != "Unknown" && !value.is_empty() {
                    self.entity_name = Some(value.to_string());
                }
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
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);

            let level = metadata.level().to_string();
            let file = metadata.file().map(|s| s.to_string());
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

    pub fn create_log_plugin(log_db: Option<PathBuf>) -> bevy::log::LogPlugin {
        let db_path = log_db.unwrap_or_else(lol_share::paths::default_log_db_path);

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).expect("无法创建日志目录");
        }

        if LOG_TX.get().is_none() {
            if db_path.exists() {
                let _ = std::fs::remove_file(&db_path);
                let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
                let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
            }

            let (tx, rx) = unbounded::<StructuredLog>();
            if LOG_TX.set(tx).is_ok() {
                std::thread::Builder::new()
                    .name("lol-core-log-writer".into())
                    .spawn(move || {
                        log_writer_main(db_path, rx);
                    })
                    .expect("无法启动日志 writer 线程");
            }
        }

        let plugin = bevy::log::LogPlugin {
            filter: "".to_owned(),
            fmt_layer: |_app| {
                let format_layer = fmt::layer()
                    .with_file(true)
                    .with_line_number(true)
                    .with_target(false)
                    .with_thread_ids(false)
                    .with_thread_names(false);

                let combined = format_layer.and_then(StructuredLogLayer);
                let is_headless = std::env::args().any(|arg| arg == "--headless");
                Some(
                    FilteredLayer {
                        inner: combined,
                        headless: is_headless,
                    }
                    .boxed(),
                )
            },
            ..default()
        };

        plugin
    }

    fn log_writer_main(db_path: PathBuf, rx: Receiver<StructuredLog>) {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("[lol_core::log] 无法创建 writer runtime: {e}");
                return;
            }
        };

        rt.block_on(async move {
            let pool = match open_and_init_db(&db_path).await {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[lol_core::log] 打开日志 DB 失败: {e}");
                    return;
                }
            };

            while let Ok(log) = rx.recv() {
                if let Err(e) = insert_log(&pool, &log).await {
                    eprintln!("[lol_core::log] 插入日志失败: {e}");
                }
            }

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
            );
            CREATE INDEX IF NOT EXISTS idx_logs_category ON logs(category);
            CREATE INDEX IF NOT EXISTS idx_logs_entity_id ON logs(entity_id);",
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
        .bind(log.entity_id.map(|id| id as i64))
        .bind(&log.entity_name)
        .bind(&log.category)
        .bind(&log.message)
        .execute(pool)
        .await?;

        Ok(())
    }
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use sqlx::Row;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    #[tokio::test]
    async fn dump_recent_logs() {
        let db_path = lol_share::paths::default_log_db_path();
        if !db_path.exists() {
            println!("No debug.db found at {:?}", db_path);
            return;
        }

        let opts = SqliteConnectOptions::new()
            .filename(&db_path)
            .read_only(true);
        let pool = SqlitePoolOptions::new().connect_with(opts).await.unwrap();

        let rows = sqlx::query(
            "SELECT id, timestamp, level, category, entity_id, message FROM logs 
             WHERE category IN ('movement', 'run', 'attack') OR level IN ('warn', 'error') OR message LIKE '%复活%' OR message LIKE '%死%'
             ORDER BY id DESC LIMIT 100"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        println!("=== FILTERED LOGS (total: {}) ===", rows.len());
        for row in rows.into_iter().rev() {
            let id: i64 = row.get("id");
            let level: String = row.get("level");
            let category: Option<String> = row.get("category");
            let entity_id: Option<i64> = row.get("entity_id");
            let msg: String = row.get("message");
            println!(
                "[#{id}] [{level}] [{:?}] entity={:?}: {msg}",
                category, entity_id
            );
        }
    }
}
