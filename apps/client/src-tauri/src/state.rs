use std::collections::HashMap;
use std::sync::{Arc, Weak};

use lol_client::WsSession;
use lol_game_process_manager::{AgentRunner, GameProcessManager};
use tauri::ipc::Channel;
use uuid::Uuid;

use crate::agent::make_desktop_runner;
use crate::process::DesktopProcessLauncher;

/// 全局应用状态：进程管理器 + 按对局 ID 的前端调试 WS 会话表。
pub struct AppState {
    pub manager: Arc<GameProcessManager>,
    /// match_id -> 前端调试 WS 会话（与 AI 决策环各自的 WS 互不干扰）。
    pub ws_sessions: Arc<std::sync::Mutex<HashMap<Uuid, WsSession>>>,
    /// match_id -> 订阅了该对局事件的前端 Channels 列表。
    pub event_channels: Arc<std::sync::Mutex<HashMap<Uuid, Vec<Channel<serde_json::Value>>>>>,
    pub model_providers: Arc<std::sync::Mutex<Vec<crate::ModelProvider>>>,
}

impl AppState {
    pub fn new(app: tauri::AppHandle) -> Self {
        let launcher = Arc::new(DesktopProcessLauncher::new(app.clone()));
        let manager = Arc::new_cyclic(|weak: &Weak<GameProcessManager>| {
            let runner: AgentRunner = make_desktop_runner(app.clone(), weak.clone());
            GameProcessManager::new(launcher.clone(), Some(runner))
        });
        Self {
            manager,
            ws_sessions: Arc::new(std::sync::Mutex::new(HashMap::new())),
            event_channels: Arc::new(std::sync::Mutex::new(HashMap::new())),
            model_providers: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
}
