use std::path::PathBuf;
use std::sync::Mutex;

use bevy::log::tracing_subscriber::{Layer, fmt};
use bevy::prelude::*;
use rusqlite::Connection;
use serde::Serialize;
use tracing::Event;
use tracing::field::{Field, Visit};

/// 全局 SQLite 连接，用于在 tracing Layer 中写入日志
static LOG_DB: Mutex<Option<Connection>> = Mutex::new(None);

/// 存储 db 文件绝对路径的 Bevy Resource
#[derive(Resource, Clone)]
pub struct LogDbPath(pub PathBuf);

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
        let line = metadata.line().map(|l| l as i64);

        let Ok(guard) = LOG_DB.lock() else {
            return;
        };
        let Some(conn) = guard.as_ref() else {
            return;
        };
        let _ = conn.execute(
            "INSERT INTO logs (timestamp, level, file, line, entity_id, entity_name, category, message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                timestamp,
                level,
                file,
                line,
                visitor.entity_id,
                visitor.entity_name,
                visitor.category,
                visitor.message,
            ],
        );
    }
}

/// 初始化 SQLite 日志数据库并返回 Bevy LogPlugin + db 路径
pub fn create_log_plugin() -> (bevy::log::LogPlugin, PathBuf) {
    let db_path = {
        let base = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        base.join(".moon-lol").join("logs").join("debug.db")
    };

    // 确保目录存在
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).expect("无法创建日志目录");
    }

    if db_path.exists() {
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
        let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
    }

    // 初始化 SQLite
    let conn = Connection::open(&db_path).expect("无法打开日志 SQLite");
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         DROP TABLE IF EXISTS logs;
         CREATE TABLE logs (
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
    .expect("无法初始化日志表");

    *LOG_DB.lock().unwrap() = Some(conn);

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
            Some(Box::new(combined))
        },
        ..Default::default()
    };

    (plugin, db_path)
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

fn on_command_log(
    event: On<CommandLog>,
    q_name: Query<Option<&Name>>,
) {
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
