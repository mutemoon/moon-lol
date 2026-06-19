use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AiConfig {
    pub api_key: String,
    pub base_url: String,
    pub preamble: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpawnPreset {
    pub name: String,
    pub x: f32,
    pub z: f32,
    pub team: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentPreset {
    pub name: String,
    pub agent_type: String,
    pub prompt: String,
    #[serde(default)]
    pub preamble: String,
    #[serde(default)]
    pub model: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeroPreset {
    pub name: String,
    pub champion: String,
    pub agent_preset_name: String,
    pub spawn_preset_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrontAgentConfig {
    pub id: Option<String>,
    pub champion: String,
    pub team: String,
    pub prompt: String,
    pub spawn_point: Vec<f32>,
    pub agent_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomScenario {
    pub name: String,
    pub agents: Vec<FrontAgentConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameConfig {
    pub mode: String,
    pub champion: String,
    pub scene_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogRow {
    pub id: i64,
    pub timestamp: i64,
    pub level: String,
    pub file: Option<String>,
    pub line: Option<i32>,
    pub entity_id: Option<i32>,
    pub entity_name: Option<String>,
    pub category: Option<String>,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueryLogsResult {
    pub rows: Vec<LogRow>,
    pub total_count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueryLogsParams {
    pub offset: i64,
    pub limit: usize,
    pub levels: Option<Vec<String>>,
    pub entity_id: Option<i32>,
    pub category: Option<String>,
    pub search_text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameHistorySummary {
    pub datetime: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterRequest {
    pub phone: String,
    pub password: String,
    pub code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginRequest {
    pub phone: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResetPasswordRequest {
    pub phone: String,
    pub new_password: String,
    pub code: String,
}
