//! History wire DTO（游戏历史记录）。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameHistorySummary {
    pub id: Option<String>,
    pub datetime: String,
    pub duration: i64,
    pub agents: Vec<AgentSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSummary {
    pub agent_id: String,
    pub champion: String,
    pub team: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedAgentHistory {
    pub agent_id: String,
    pub champion: String,
    pub team: String,
    pub prompt: String,
    pub system_prompt: String,
    pub history: Vec<serde_json::Value>,
    pub game_duration: i64,
    pub datetime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadHistoryRequest {
    pub histories: Vec<SavedAgentHistory>,
}
