use async_trait::async_trait;
use crate::models::{
    AiConfig, SpawnPreset, AgentPreset, HeroPreset, FrontAgentConfig,
    GameConfig, QueryLogsParams, QueryLogsResult
};

#[async_trait]
pub trait ConfigService: Send + Sync {
    async fn get_ai_config(&self, user_id: i32) -> Result<AiConfig, String>;
    async fn set_ai_config(&self, user_id: i32, config: AiConfig) -> Result<(), String>;
}

#[async_trait]
pub trait PresetService: Send + Sync {
    // Spawn Presets
    async fn list_spawn_presets(&self, user_id: i32) -> Result<Vec<SpawnPreset>, String>;
    async fn save_spawn_preset(&self, user_id: i32, preset: SpawnPreset) -> Result<(), String>;
    async fn delete_spawn_preset(&self, user_id: i32, name: &str) -> Result<(), String>;

    // Agent Presets
    async fn list_agent_presets(&self, user_id: i32) -> Result<Vec<AgentPreset>, String>;
    async fn save_agent_preset(&self, user_id: i32, preset: AgentPreset) -> Result<(), String>;
    async fn delete_agent_preset(&self, user_id: i32, name: &str) -> Result<(), String>;

    // Hero Presets
    async fn list_hero_presets(&self, user_id: i32) -> Result<Vec<HeroPreset>, String>;
    async fn save_hero_preset(&self, user_id: i32, preset: HeroPreset) -> Result<(), String>;
    async fn delete_hero_preset(&self, user_id: i32, name: &str) -> Result<(), String>;
}

#[async_trait]
pub trait ScenarioService: Send + Sync {
    async fn list_custom_scenarios(&self, user_id: i32) -> Result<Vec<String>, String>;
    async fn load_custom_scenario(&self, user_id: i32, scene_name: &str) -> Result<Vec<FrontAgentConfig>, String>;
    async fn save_custom_scenario(&self, user_id: i32, scene_name: &str, agents: Vec<FrontAgentConfig>) -> Result<(), String>;
    async fn delete_custom_scenario(&self, user_id: i32, scene_name: &str) -> Result<(), String>;
    async fn load_scenario_win_condition(&self, user_id: i32, scene_name: &str) -> Result<serde_json::Value, String>;
    async fn save_scenario_win_condition(&self, user_id: i32, scene_name: &str, condition: serde_json::Value) -> Result<(), String>;
}

#[async_trait]
pub trait GameService: Send + Sync {
    async fn start_game(&self, user_id: i32, config: GameConfig) -> Result<(), String>;
    async fn stop_game(&self) -> Result<(), String>;
    async fn get_active_game_port(&self) -> Result<Option<u16>, String>;
}

#[async_trait]
pub trait HistoryService: Send + Sync {
    async fn list_game_histories(&self, user_id: i32) -> Result<Vec<serde_json::Value>, String>;
    async fn get_game_history_detail(&self, user_id: i32, datetime: &str) -> Result<Vec<serde_json::Value>, String>;
    async fn delete_game_history(&self, user_id: i32, datetime: &str) -> Result<(), String>;
}

#[async_trait]
pub trait LogService: Send + Sync {
    async fn query_log_entities(&self, user_id: i32) -> Result<Vec<serde_json::Value>, String>;
    async fn query_log_categories(&self, user_id: i32) -> Result<Vec<serde_json::Value>, String>;
    async fn query_logs(&self, user_id: i32, params: QueryLogsParams) -> Result<QueryLogsResult, String>;
    async fn clear_logs(&self, user_id: i32) -> Result<(), String>;
}

#[async_trait]
pub trait UserService: Send + Sync {
    async fn register(&self, phone: &str, password: &str, code: &str) -> Result<serde_json::Value, String>;
    async fn login(&self, phone: &str, password: &str) -> Result<serde_json::Value, String>;
    async fn reset_password(&self, phone: &str, new_password: &str, code: &str) -> Result<(), String>;
}
