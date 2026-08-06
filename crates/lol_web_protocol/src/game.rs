//! 本地对局域 wire DTO（桌面端与 cloud 共用）。
//!
//! 注意：RunningGame 使用 camelCase（对齐 Tauri 命令返回与 types.ts）。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontAgentConfig {
    pub id: Option<String>,
    pub champion: String,
    pub team: String,
    pub prompt: String,
    pub spawn_point: Vec<f32>,
    #[serde(default = "default_agent_type")]
    pub agent_type: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub provider_id: Option<String>,
    #[serde(default)]
    pub config_json: Option<serde_json::Value>,
}

fn default_agent_type() -> String {
    "llm".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub mode: String,
    pub champion: String,
    pub scene_name: Option<String>,
    #[serde(default)]
    pub agents: Option<Vec<FrontAgentConfig>>,
    #[serde(default)]
    pub providers: Option<Vec<super::model_provider::ModelProvider>>,
}

/// 运行中的对局摘要（camelCase，对齐前端 RunningGame 接口）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningGame {
    pub id: String,
    pub port: i32,
    pub status: String,
}
