//! 本地专用类型：仅保留共享协议 `lol_web_protocol` 未覆盖的类型。
//!
//! 跨进程 DTO（GameConfig / FrontAgentConfig / RunningGame / ModelProvider）已迁移到
//! `lol_web_protocol` 并在此处通过 re-export 引用，本文件不再定义副本。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lol_client::WsSession;
use tokio::sync::mpsc;
use uuid::Uuid;

// ── 日志类型（共享协议未覆盖，log_service 专用）──

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct LogRow {
    pub id: i64,
    pub timestamp: i64,
    pub level: String,
    pub file: Option<String>,
    pub line: Option<i64>,
    pub entity_id: Option<i64>,
    pub entity_name: Option<String>,
    pub category: Option<String>,
    pub message: String,
}

#[derive(serde::Serialize, Clone)]
pub struct QueryLogsResult {
    pub rows: Vec<LogRow>,
    pub total_count: i64,
}

#[derive(serde::Serialize, Clone)]
pub struct LogEntity {
    pub entity_id: Option<i64>,
    pub entity_name: Option<String>,
}

#[derive(serde::Serialize, Clone)]
pub struct LogCategory {
    pub category: Option<String>,
}

/// 日志查询参数（对齐前端 `LogQueryParams`）。
#[derive(Debug, Clone)]
pub struct LogQueryParams {
    pub offset: i64,
    pub limit: i64,
    pub levels: Option<Vec<String>>,
    pub entity_id: Option<i64>,
    pub category: Option<String>,
    pub search_text: Option<String>,
}

// ── 共享内部状态 ──

/// WS 会话表：match_id → 调试 WS 会话。
pub type WsSessionMap = Arc<Mutex<HashMap<Uuid, WsSession>>>;
/// 事件通道表：match_id → 订阅者列表。
pub type EventChannelMap = Arc<Mutex<HashMap<Uuid, Vec<mpsc::Sender<serde_json::Value>>>>>;

/// 本地游戏共享状态（进程 / 会话 / 事件通道）。
pub struct LocalGameState {
    pub ws_sessions: WsSessionMap,
    pub event_channels: EventChannelMap,
}

impl LocalGameState {
    pub fn new() -> Self {
        Self {
            ws_sessions: Arc::new(Mutex::new(HashMap::new())),
            event_channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
