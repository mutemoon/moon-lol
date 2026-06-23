//! Scenario 子系统的领域层（场景预设：完整阵容 + 可选胜利条件）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Scenario（"场景预设"）：一组完整阵容编排 + 可选胜利条件树。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scenario {
    pub id: Uuid,
    pub owner_id: i32,
    pub name: String,
    /// 完整阵容编排（FrontAgentConfig 数组，结构由前端契约定义）。
    pub agents: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// 创建 / 更新输入。created_at 由 DB 生成。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioInput {
    pub name: String,
    pub agents: serde_json::Value,
}

/// 名称校验：非空 trim 后、不超过 64 字符。
pub fn validate_name(name: &str) -> bool {
    let n = name.trim();
    !n.is_empty() && n.len() <= 64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_name_accepts_normal() {
        assert!(validate_name("5v5 激进阵容"));
        assert!(validate_name("x"));
    }

    #[test]
    fn validate_name_rejects_empty_or_whitespace() {
        assert!(!validate_name(""));
        assert!(!validate_name("   "));
    }

    #[test]
    fn validate_name_rejects_too_long() {
        assert!(!validate_name(&"x".repeat(65)));
        assert!(validate_name(&"x".repeat(64)));
    }

    #[test]
    fn scenario_serde_roundtrip() {
        let s = Scenario {
            id: Uuid::new_v4(),
            owner_id: 1,
            name: "阵容A".into(),
            agents: serde_json::json!([{"champion": "Riven"}]),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: Scenario = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
