//! Agent wire DTO（"选手" = 英雄 + 配置）。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::spawn_preset::Visibility;

// ── AgentType 枚举 ──

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

// ── Agent DTO ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    pub id: Uuid,
    pub owner_id: i32,
    pub name: String,
    pub champion: String,
    pub agent_type: AgentType,
    pub prompt: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub config_json: Option<serde_json::Value>,
    pub visibility: Visibility,
    pub forked_from: Option<Uuid>,
    pub upstream_agent_id: Option<Uuid>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentDto {
    pub name: String,
    pub champion: String,
    pub agent_type: AgentType,
    pub prompt: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub config_json: Option<serde_json::Value>,
    #[serde(default)]
    pub visibility: Option<Visibility>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateAgentDto {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub champion: Option<String>,
    #[serde(default)]
    pub agent_type: Option<AgentType>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub config_json: Option<serde_json::Value>,
    #[serde(default)]
    pub visibility: Option<Visibility>,
}

// ── roundtrip 单测 ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_type_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&AgentType::Llm).unwrap(), r#""llm""#);
        assert_eq!(serde_json::to_string(&AgentType::Rl).unwrap(), r#""rl""#);
        assert_eq!(
            serde_json::to_string(&AgentType::Script).unwrap(),
            r#""script""#
        );
    }

    #[test]
    fn agent_type_roundtrip() {
        let cases = ["llm", "rl", "script"];
        for s in cases {
            let t: AgentType = serde_json::from_str(&format!(r#""{s}""#)).unwrap();
            assert_eq!(serde_json::to_string(&t).unwrap(), format!(r#""{s}""#));
        }
    }

    #[test]
    fn agent_deserializes_missing_optional_fields() {
        let json = r#"{"id":"550e8400-e29b-41d4-a716-446655440000","owner_id":1,"name":"test","champion":"Riven","agent_type":"llm","prompt":"aggro","visibility":"public","forked_from":null,"upstream_agent_id":null,"created_at":"2025-01-01T00:00:00Z","updated_at":"2025-01-01T00:00:00Z"}"#;
        let agent: Agent = serde_json::from_str(json).unwrap();
        assert_eq!(agent.model, None);
        assert_eq!(agent.config_json, None);
    }
}
