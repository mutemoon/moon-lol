use std::collections::HashMap;

use lol_rl_protocol::{
    CheckpointItem, MetricsRow, ObsFeaturePayload, PolicyItem, RewardFormulaSpec, RewardItem,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Home,
    Launcher,
    Heroes,
    Rooms,
    Rank,
    Leaderboard,
    Community,
    Billing,
    RlTraining,
    Particles,
    LogsArchive,
    Admin,
    Settings,
    Games,
    History,
    Blog,
    Debug,
    Mock,
    Observe,
    RoomDetail,
    Hero,
    RlTaskDetail,
    VisualEnv,
    WadBrowser,
    Extractor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskDetailTab {
    Metrics,
    Models,
    VisualEnv,
}

#[derive(Debug, Clone)]
pub struct LocalTaskDetail {
    pub id: String,
    pub name: String,
    pub agent_type: String,
    pub env_name: String,
    pub status: String,
    pub current_step: usize,
    pub ep_return: f32,
    pub checkpoints: Vec<CheckpointItem>,
    pub metrics_history: Vec<MetricsRow>,
    pub latest_policy: Vec<PolicyItem>,
    pub latest_reward_breakdown: Vec<RewardItem>,
    pub latest_obs: Option<ObsFeaturePayload>,
    pub reward_formula: Option<RewardFormulaSpec>,
    pub latest_reward_variables: Option<HashMap<String, f32>>,
    pub logs: Vec<String>,
}

// ── 全局状态辅助类型（对应 Vue Pinia stores，M3/M4 填充数据流） ──

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub id: i64,
    pub phone: String,
}

#[derive(Debug, Clone)]
pub struct RunningGameInfo {
    pub id: String,
    pub mode: String,
    pub champion: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct ModelProviderInfo {
    pub id: Option<String>,
    pub name: String,
    pub category: String,
    pub models: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct SpawnPreset {
    pub name: String,
    pub x: f32,
    pub z: f32,
    pub team: String,
}

#[derive(Debug, Clone)]
pub struct HeroPreset {
    pub name: String,
    pub hero: String,
    pub agent_type: String,
}
