//! 英雄/选手页辅助纯函数：展示文本 / config_json 组装 / 导入导出。

use lol_web_protocol::agent::{Agent, AgentType};
use lol_web_protocol::agent_snapshot::AgentSnapshot;
use lol_web_protocol::spawn_preset::Visibility;

use super::types::{HeroesState, PLATFORM_PROVIDER_ID};

pub(super) fn visibility_label(v: Visibility) -> &'static str {
    match v {
        Visibility::Private => "私有",
        Visibility::Friends => "好友可见",
        Visibility::Public => "公开",
    }
}

pub(super) fn latest_snapshot_label(snaps: &[AgentSnapshot]) -> String {
    snaps
        .first()
        .map_or_else(|| "未发布".to_string(), |s| format!("v{}", s.version))
}

pub(super) fn has_unpublished_changes(agent: &Agent, snaps: &[AgentSnapshot]) -> bool {
    let latest = match snaps.first() {
        Some(s) => s,
        None => return true,
    };
    agent.updated_at.as_str() > latest.created_at.as_str()
}

pub(super) fn ago(iso: &str) -> String {
    iso.chars().take(10).collect()
}

pub(super) fn champion_display(name: &str) -> String {
    let key = format!("champions.{}", name);
    let localized = rust_i18n::t!(&key);
    if localized != key {
        localized.to_string()
    } else {
        name.to_string()
    }
}

pub(super) fn cfg_str(cfg: &Option<serde_json::Value>, key: &str) -> String {
    cfg.as_ref()
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

pub(super) fn pretty_config(cfg: &Option<serde_json::Value>) -> String {
    cfg.as_ref().map_or_else(
        || "{}".to_string(),
        |v| serde_json::to_string_pretty(v).unwrap_or_else(|_| "{}".to_string()),
    )
}

/// 当前草稿的 config_json（按类型组装，参照 heroes.vue handleSave）。
pub(super) fn draft_config(state: &HeroesState) -> serde_json::Value {
    match state.draft_agent_type {
        AgentType::Llm => {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "thinking_depth".into(),
                serde_json::json!(state.draft_thinking_depth),
            );
            if !state.draft_provider_id.is_empty()
                && state.draft_provider_id != PLATFORM_PROVIDER_ID
            {
                obj.insert(
                    "provider_id".into(),
                    serde_json::json!(state.draft_provider_id),
                );
            }
            serde_json::Value::Object(obj)
        }
        AgentType::Rl => {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "model_path".into(),
                serde_json::json!(state.draft_rl_model_path),
            );
            obj.insert(
                "inference_endpoint".into(),
                serde_json::json!(state.draft_rl_endpoint),
            );
            let mut rs = serde_json::Map::new();
            for (k, v) in &state.draft_rl_rewards {
                rs.insert(k.clone(), serde_json::json!(v));
            }
            obj.insert("reward_shaper".into(), serde_json::Value::Object(rs));
            serde_json::Value::Object(obj)
        }
        AgentType::Script => serde_json::json!({ "script": state.draft_script }),
    }
}

/// 导出用整包 JSON（agent 配置），供展示/复制/导入。
pub(super) fn export_json(state: &HeroesState) -> String {
    let model = if state.draft_model.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(state.draft_model.clone())
    };
    serde_json::to_string_pretty(&serde_json::json!({
        "name": state.draft_name,
        "champion": state.draft_champion,
        "agent_type": state.draft_agent_type.as_str(),
        "prompt": state.draft_prompt,
        "model": model,
        "config_json": draft_config(state),
    }))
    .unwrap_or_default()
}

/// 把粘贴/输入的 JSON 填充进草稿字段。
pub(super) fn apply_import_json(state: &mut HeroesState, s: &str) -> Result<(), String> {
    let v: serde_json::Value = serde_json::from_str(s).map_err(|e| e.to_string())?;
    let obj = v.as_array().and_then(|a| a.first()).unwrap_or(&v);
    let obj = obj
        .as_object()
        .ok_or_else(|| "期望 JSON 对象".to_string())?;

    if let Some(n) = obj.get("name").and_then(|v| v.as_str()) {
        state.draft_name = n.to_string();
    }
    if let Some(c) = obj.get("champion").and_then(|v| v.as_str()) {
        state.draft_champion = c.to_string();
    }
    if let Some(t) = obj
        .get("agent_type")
        .and_then(|v| v.as_str())
        .and_then(AgentType::from_str)
    {
        state.draft_agent_type = t;
    }
    if let Some(p) = obj.get("prompt").and_then(|v| v.as_str()) {
        state.draft_prompt = p.to_string();
    }
    if let Some(m) = obj.get("model").and_then(|v| v.as_str()) {
        state.draft_model = m.to_string();
    }
    if let Some(cfg) = obj.get("config_json").and_then(|v| v.as_object()) {
        if let Some(td) = cfg.get("thinking_depth").and_then(|v| v.as_u64()) {
            state.draft_thinking_depth = td as u32;
        }
        if let Some(pid) = cfg.get("provider_id").and_then(|v| v.as_str()) {
            state.draft_provider_id = pid.to_string();
        }
        if let Some(p) = cfg.get("model_path").and_then(|v| v.as_str()) {
            state.draft_rl_model_path = p.to_string();
        }
        if let Some(e) = cfg.get("inference_endpoint").and_then(|v| v.as_str()) {
            state.draft_rl_endpoint = e.to_string();
        }
        if let Some(rs) = cfg.get("reward_shaper").and_then(|v| v.as_object()) {
            for (k, v) in rs {
                if let Some(n) = v.as_f64() {
                    state.draft_rl_rewards.insert(k.clone(), n);
                }
            }
        }
        if let Some(sc) = cfg.get("script").and_then(|v| v.as_str()) {
            state.draft_script = sc.to_string();
        }
    }
    Ok(())
}

/// Agent 的文本快照（prompt + config），用于 Fork diff 两栏对照。
pub(super) fn pretty_agent(a: &Agent) -> String {
    format!(
        "名称: {}\n英雄: {}\n类型: {}\n模型: {}\n\n【Prompt】\n{}\n\n【配置 JSON】\n{}",
        a.name,
        a.champion,
        a.agent_type.as_str(),
        a.model.as_deref().unwrap_or("(默认)"),
        a.prompt,
        pretty_config(&a.config_json),
    )
}
