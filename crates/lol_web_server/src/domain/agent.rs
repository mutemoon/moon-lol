//! Agent 子系统的领域层（"选手" = 英雄 + 配置 + 出生点）。

use serde::{Deserialize, Serialize};

use super::spawn_preset::Visibility;

/// Agent（"选手"）：参赛/ELO/排行榜主体。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    pub id: uuid::Uuid,
    pub owner_id: i32,
    pub name: String,
    pub champion: String,
    pub agent_config_id: uuid::Uuid,
    pub spawn_preset_id: Option<uuid::Uuid>,
    pub visibility: Visibility,
    pub forked_from: Option<uuid::Uuid>,
    pub upstream_agent_id: Option<uuid::Uuid>,
}

/// 创建输入。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    pub name: String,
    pub champion: String,
    pub agent_config_id: uuid::Uuid,
    pub spawn_preset_id: Option<uuid::Uuid>,
    pub visibility: Visibility,
}

/// 默认 Agent 槽位上限（免费用户）。
pub const DEFAULT_AGENT_LIMIT: usize = 5;

/// 槽位限制校验：当前数量是否已达上限。
pub fn assert_within_slot_limit(current: usize, limit: usize) -> Result<(), SlotLimitError> {
    if current >= limit {
        Err(SlotLimitError { current, limit })
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlotLimitError {
    pub current: usize,
    pub limit: usize,
}

/// 英雄名校验：非空，不超过 32 字符。
pub fn validate_champion(champion: &str) -> bool {
    let c = champion.trim();
    !c.is_empty() && c.len() <= 32
}

pub fn validate_name(name: &str) -> bool {
    let n = name.trim();
    !n.is_empty() && n.len() <= 64
}

/// 可见性判定（与 AgentConfig 的敏感字段判定不同：Agent 基础信息对 public 可见，
/// 但不暴露 prompt/model —— 那些在 agent_config 上）。
pub fn can_view(agent: &Agent, requester_id: i32) -> bool {
    if agent.owner_id == requester_id {
        return true;
    }
    matches!(agent.visibility, Visibility::Public | Visibility::Friends)
}

/// Fork 一个 Agent 时，新 Agent 的默认命名。
pub fn fork_name(original_name: &str) -> String {
    format!("{original_name} · 副本")
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn sample_agent(owner: i32, vis: Visibility) -> Agent {
        Agent {
            id: Uuid::new_v4(),
            owner_id: owner,
            name: "锐雯 · 激进".into(),
            champion: "Riven".into(),
            agent_config_id: Uuid::new_v4(),
            spawn_preset_id: Some(Uuid::new_v4()),
            visibility: vis,
            forked_from: None,
            upstream_agent_id: None,
        }
    }

    #[test]
    fn slot_limit_allows_below_cap() {
        assert!(assert_within_slot_limit(3, 5).is_ok());
        assert!(assert_within_slot_limit(0, 5).is_ok());
    }

    #[test]
    fn slot_limit_blocks_at_cap() {
        let err = assert_within_slot_limit(5, 5).unwrap_err();
        assert_eq!(err.current, 5);
        assert_eq!(err.limit, 5);
    }

    #[test]
    fn slot_limit_blocks_above_cap() {
        assert!(assert_within_slot_limit(6, 5).is_err());
    }

    #[test]
    fn champion_validation() {
        assert!(validate_champion("Riven"));
        assert!(!validate_champion(""));
        assert!(!validate_champion(&"x".repeat(33)));
    }

    #[test]
    fn owner_always_can_view() {
        for vis in [Visibility::Private, Visibility::Friends, Visibility::Public] {
            let agent = sample_agent(1, vis);
            assert!(can_view(&agent, 1));
        }
    }

    #[test]
    fn non_owner_cannot_view_private() {
        let agent = sample_agent(1, Visibility::Private);
        assert!(!can_view(&agent, 2));
    }

    #[test]
    fn non_owner_can_view_public_and_friends() {
        let pub_agent = sample_agent(1, Visibility::Public);
        let friend_agent = sample_agent(1, Visibility::Friends);
        assert!(can_view(&pub_agent, 2));
        assert!(can_view(&friend_agent, 2));
    }

    #[test]
    fn fork_name_appends_suffix() {
        assert_eq!(fork_name("锐雯 · 激进"), "锐雯 · 激进 · 副本");
        assert_eq!(fork_name("X"), "X · 副本");
    }
}
