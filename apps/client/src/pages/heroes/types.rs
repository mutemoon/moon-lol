//! 英雄/选手页状态类型（存储在 `AppSidebar.heroes`）。

use std::collections::HashMap;

use lol_web_protocol::agent::{Agent, AgentType};
use lol_web_protocol::agent_snapshot::AgentSnapshot;
use lol_web_protocol::model_provider::ModelProvider;
use lol_web_protocol::spawn_preset::Visibility;
use uuid::Uuid;

/// 平台模型供应商 id（对应 client 的 PLATFORM_PROVIDER_ID = "__platform__"）。
pub(super) const PLATFORM_PROVIDER_ID: &str = "__platform__";

/// RL Reward Shaper 固定权重键（对应 heroes.vue 的 RL_REWARD_KEYS）。
pub(super) const RL_REWARD_KEYS: [&str; 9] = [
    "last_hit",
    "kill",
    "death",
    "assist",
    "gold",
    "level",
    "health",
    "time",
    "proximity",
];

#[derive(Clone, PartialEq)]
pub enum HeroesMode {
    Browse,
    Edit { editing_id: Option<Uuid> },
}

#[derive(Clone, Copy, PartialEq)]
pub enum HeroesTab {
    Config,
    Publish,
}

pub struct HeroesState {
    pub mode: HeroesMode,
    pub agents: Vec<Agent>,
    pub snapshots: HashMap<Uuid, Vec<AgentSnapshot>>,
    pub upstream_agent: Option<Agent>,
    pub loading: bool,
    pub error_msg: String,
    pub success_msg: String,
    pub show_delete_confirm: bool,
    pub deleting: bool,

    pub draft_name: String,
    pub draft_champion: String,
    pub draft_agent_type: AgentType,
    pub draft_prompt: String,
    pub draft_model: String,
    pub draft_config_json_str: String,
    pub draft_visibility: Visibility,
    pub draft_thinking_depth: u32,
    pub draft_provider_id: String,
    pub draft_manual_model: bool,
    pub draft_rl_model_path: String,
    pub draft_rl_endpoint: String,
    pub draft_rl_rewards: HashMap<String, f64>,
    pub draft_script: String,
    pub selected_tab: HeroesTab,
    pub publishing: bool,

    pub platform_models: Vec<String>,
    pub model_providers: Vec<ModelProvider>,
    pub providers_loaded: bool,
}

/// RL Reward Shaper 默认权重。
pub(super) fn default_rewards() -> HashMap<String, f64> {
    let mut m = HashMap::new();
    m.insert("last_hit".into(), 1.0);
    m.insert("kill".into(), 5.0);
    m.insert("death".into(), -5.0);
    m.insert("assist".into(), 2.0);
    m.insert("gold".into(), 0.0);
    m.insert("level".into(), 1.0);
    m.insert("health".into(), 1.0);
    m.insert("time".into(), -0.001);
    m.insert("proximity".into(), 0.0);
    m
}

impl Default for HeroesState {
    fn default() -> Self {
        Self {
            mode: HeroesMode::Browse,
            agents: Vec::new(),
            snapshots: HashMap::new(),
            upstream_agent: None,
            loading: false,
            error_msg: String::new(),
            success_msg: String::new(),
            show_delete_confirm: false,
            deleting: false,
            draft_name: String::new(),
            draft_champion: "Riven".to_string(),
            draft_agent_type: AgentType::Llm,
            draft_prompt: String::new(),
            draft_model: String::new(),
            draft_config_json_str: String::new(),
            draft_visibility: Visibility::Private,
            draft_thinking_depth: 2,
            draft_provider_id: PLATFORM_PROVIDER_ID.to_string(),
            draft_manual_model: false,
            draft_rl_model_path: String::new(),
            draft_rl_endpoint: String::new(),
            draft_rl_rewards: default_rewards(),
            draft_script: String::new(),
            selected_tab: HeroesTab::Config,
            publishing: false,
            platform_models: Vec::new(),
            model_providers: Vec::new(),
            providers_loaded: false,
        }
    }
}
