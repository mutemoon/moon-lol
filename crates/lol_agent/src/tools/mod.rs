use bevy::prelude::*;
use lol_core::action::Action;
use rand::Rng;
use rig::client::CompletionClient;
use rig::completion::Chat;

use crate::models::Observe;

pub mod combat;
pub mod level;
pub mod movement;
pub mod prompt;

pub use combat::*;
pub use level::*;
pub use movement::*;
pub use prompt::*;

// 全局共享静态状态
pub static GLOBAL_ACTION: std::sync::Mutex<Option<Action>> = std::sync::Mutex::new(None);
pub static GLOBAL_MINION_ENTITIES: std::sync::Mutex<Vec<Entity>> =
    std::sync::Mutex::new(Vec::new());
pub static GLOBAL_HERO_ENTITY: std::sync::Mutex<Option<Entity>> = std::sync::Mutex::new(None);
pub static GLOBAL_CHAT_HISTORY: std::sync::Mutex<Vec<rig::completion::Message>> =
    std::sync::Mutex::new(Vec::new());

static GLOBAL_AGENT: std::sync::OnceLock<
    rig::agent::Agent<rig::providers::anthropic::completion::GenericCompletionModel>,
> = std::sync::OnceLock::new();

// 硬编码配置 Anthropic API Key 和代理服务的 Base URL
const HARDCODED_ANTHROPIC_API_KEY: &str = "";
const HARDCODED_ANTHROPIC_BASE_URL: &str = "https://api.deepseek.com/anthropic";

use crate::models::AgentDecisionResult;

pub async fn call_rig_llm(observe: Observe) -> AgentDecisionResult {
    let mut api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    let mut base_url = std::env::var("ANTHROPIC_BASE_URL").unwrap_or_default();

    if api_key.is_empty() {
        api_key = HARDCODED_ANTHROPIC_API_KEY.to_string();
    }
    if base_url.is_empty() {
        base_url = HARDCODED_ANTHROPIC_BASE_URL.to_string();
    }

    if api_key.is_empty() || api_key == "YOUR_ANTHROPIC_API_KEY" {
        warn!(
            "未检测到有效的 ANTHROPIC_API_KEY（环境变量或硬编码），AI Actor 将降级为使用本地随机策略决策。"
        );
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let action = fallback_random_action(&observe);
        return AgentDecisionResult {
            observe,
            thinking: "未检测到有效的 API Key，降级使用本地随机策略决策中。".to_string(),
            action,
        };
    }

    // 更新全局可观测实体状态
    {
        let mut minion_lock = GLOBAL_MINION_ENTITIES.lock().unwrap();
        *minion_lock = observe.minions.iter().map(|m| m.entity).collect();

        let mut hero_lock = GLOBAL_HERO_ENTITY.lock().unwrap();
        *hero_lock = observe.enemy_hero.as_ref().map(|h| h.entity);

        let mut action_lock = GLOBAL_ACTION.lock().unwrap();
        *action_lock = None;
    }

    // 初始化全局共享 Agent 客户端
    let agent = GLOBAL_AGENT.get_or_init(|| {
        let client = rig::providers::anthropic::Client::builder()
            .api_key(&api_key)
            .base_url(&base_url)
            .build()
            .expect("初始化 rig Anthropic 客户端失败");

        client
            .agent("deepseek-v4-flash")
            .max_tokens(200 * 1000)
            .preamble(SYSTEM_PROMPT)
            .tool(MoveTool)
            .tool(AttackMinionTool)
            .tool(AttackHeroTool)
            .tool(SkillTool)
            .tool(SkillLevelUpTool)
            .tool(StopTool)
            .build()
    });

    let prompt = build_prompt(&observe);

    if let Ok(json_str) = serde_json::to_string(&observe) {
        info!("[AGENT_OBSERVE] {}", json_str);
    }

    let mut local_chat_history = {
        let lock = GLOBAL_CHAT_HISTORY.lock().unwrap();
        lock.clone()
    };

    let response = agent.chat(&prompt, &mut local_chat_history);

    let thinking;
    let mut act = None;

    match response.await {
        Ok(response_text) => {
            thinking = response_text.clone();
            info!("[AGENT_RESPONSE] {}", response_text);
            info!("Rig LLM 决策工具调用文本响应成功: {}", response_text);
            let mut lock = GLOBAL_CHAT_HISTORY.lock().unwrap();
            *lock = local_chat_history;

            let mut lock = GLOBAL_ACTION.lock().unwrap();
            act = lock.take();
        }
        Err(e) => {
            error!("Rig Agent 交互请求失败: {:?}", e);
            thinking = format!("Rig Agent 交互请求失败: {:?}", e);
        }
    }

    if let Some(ref action) = act {
        info!("[AGENT_ACTION] {:?}", action);
    }

    AgentDecisionResult {
        observe,
        thinking,
        action: act,
    }
}

fn fallback_random_action(observe: &Observe) -> Option<Action> {
    let mut r = rand::rng();
    let action_type = r.random_range(0..4);
    match action_type {
        0 => {
            let offset = Vec2::new(r.random_range(-300.0..300.0), r.random_range(-300.0..300.0));
            Some(Action::Move(observe.myself.position + offset))
        }
        1 => get_fallback_attack(observe),
        2 => get_fallback_skill(observe, &mut r),
        _ => Some(Action::Stop),
    }
}

fn get_fallback_attack(observe: &Observe) -> Option<Action> {
    if !observe.minions.is_empty() {
        return Some(Action::Attack(observe.minions[0].entity));
    }
    let hero = observe.enemy_hero.as_ref()?;
    Some(Action::Attack(hero.entity))
}

fn get_fallback_skill(observe: &Observe, r: &mut rand::rngs::ThreadRng) -> Option<Action> {
    if !observe.minions.is_empty() {
        let offset = Vec2::new(r.random_range(-50.0..50.0), r.random_range(-50.0..50.0));
        return Some(Action::Skill {
            index: r.random_range(0..4),
            point: observe.minions[0].position + offset,
        });
    }
    let hero = observe.enemy_hero.as_ref()?;
    let offset = Vec2::new(r.random_range(-50.0..50.0), r.random_range(-50.0..50.0));
    Some(Action::Skill {
        index: r.random_range(0..4),
        point: hero.position + offset,
    })
}
