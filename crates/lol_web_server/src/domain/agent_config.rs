//! AgentConfig 子系统的领域层（"策略大脑"，英雄无关）。

use serde::{Deserialize, Serialize};

use super::spawn_preset::Visibility;

/// Agent 类型：LLM / RL / Script。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Llm,
    Rl,
    Script,
}

impl AgentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentType::Llm => "llm",
            AgentType::Rl => "rl",
            AgentType::Script => "script",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "llm" => Some(AgentType::Llm),
            "rl" => Some(AgentType::Rl),
            "script" => Some(AgentType::Script),
            _ => None,
        }
    }
}

/// Agent 配置（"策略大脑"）：英雄无关，可被多个 Agent 引用。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    pub id: uuid::Uuid,
    pub owner_id: i32,
    pub name: String,
    pub agent_type: AgentType,
    pub prompt: String,
    pub preamble: String,
    pub model: String,
    pub config_json: serde_json::Value,
    pub visibility: Visibility,
    pub forked_from: Option<uuid::Uuid>,
}

/// 创建输入。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfigInput {
    pub name: String,
    pub agent_type: AgentType,
    pub prompt: String,
    pub preamble: String,
    pub model: String,
    pub config_json: serde_json::Value,
    pub visibility: Visibility,
}

/// 敏感字段（prompt/model/config_json）是否对请求者可见。
///
/// 规则（PRODUCT.md §2.4 + §3.3.3）：
/// - owner 永远可见
/// - private：仅 owner
/// - friends：owner + 好友（好友系统未实现前，等同 private）
/// - public：所有人可见基础信息；敏感字段仅在 friends 或显式授权时返回
pub fn can_view_sensitive(config: &AgentConfig, requester_id: i32, is_friend: bool) -> bool {
    if config.owner_id == requester_id {
        return true;
    }
    match config.visibility {
        Visibility::Private => false,
        Visibility::Friends => is_friend,
        Visibility::Public => is_friend, // public 基础可见，但敏感字段仍需 friend
    }
}

pub fn validate_name(name: &str) -> bool {
    let n = name.trim();
    !n.is_empty() && n.len() <= 64
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn sample_config(owner: i32, vis: Visibility) -> AgentConfig {
        AgentConfig {
            id: Uuid::new_v4(),
            owner_id: owner,
            name: "激进压制".into(),
            agent_type: AgentType::Llm,
            prompt: "be aggressive".into(),
            preamble: "".into(),
            model: "claude".into(),
            config_json: serde_json::json!({}),
            visibility: vis,
            forked_from: None,
        }
    }

    #[test]
    fn agent_type_roundtrip() {
        assert_eq!(AgentType::from_str("llm"), Some(AgentType::Llm));
        assert_eq!(AgentType::from_str("SCRIPT"), Some(AgentType::Script));
        assert_eq!(AgentType::from_str("x"), None);
    }

    #[test]
    fn owner_can_view_sensitive_regardless_of_visibility() {
        for vis in [Visibility::Private, Visibility::Friends, Visibility::Public] {
            let cfg = sample_config(1, vis);
            assert!(
                can_view_sensitive(&cfg, 1, false),
                "owner 应可见敏感字段 (vis={vis:?})"
            );
        }
    }

    #[test]
    fn non_owner_cannot_view_sensitive_when_private() {
        let cfg = sample_config(1, Visibility::Private);
        assert!(!can_view_sensitive(&cfg, 2, false));
        assert!(!can_view_sensitive(&cfg, 2, true)); // 即使是好友
    }

    #[test]
    fn non_owner_friend_can_view_sensitive_when_friends() {
        let cfg = sample_config(1, Visibility::Friends);
        assert!(!can_view_sensitive(&cfg, 2, false)); // 非好友
        assert!(can_view_sensitive(&cfg, 2, true)); // 好友
    }

    #[test]
    fn non_owner_only_friend_can_view_sensitive_when_public() {
        let cfg = sample_config(1, Visibility::Public);
        assert!(!can_view_sensitive(&cfg, 2, false)); // 公开但非好友，敏感字段不可见
        assert!(can_view_sensitive(&cfg, 2, true)); // 公开且好友，可见
    }

    #[test]
    fn name_validation() {
        assert!(validate_name("激进压制"));
        assert!(!validate_name(""));
        assert!(!validate_name(&"x".repeat(65)));
    }
}
