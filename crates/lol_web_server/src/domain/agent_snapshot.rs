//! AgentSnapshot 子系统的领域层（参赛快照，Rank 队列用）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 参赛快照：Agent 某版本配置的不可变冻结。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentSnapshot {
    pub id: uuid::Uuid,
    pub agent_id: uuid::Uuid,
    pub version: i32,
    pub config_freeze: serde_json::Value,
    pub published_at: DateTime<Utc>,
}

/// 计算下一个版本号。None→1, Some(max)→max+1。
pub fn next_version(current_max: Option<i32>) -> i32 {
    match current_max {
        None => 1,
        Some(max) => max + 1,
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    fn sample_snapshot() -> AgentSnapshot {
        AgentSnapshot {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            version: 1,
            config_freeze: serde_json::json!({"champion": "Riven"}),
            published_at: DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        }
    }

    #[test]
    fn next_version_none_starts_at_one() {
        assert_eq!(next_version(None), 1);
    }

    #[test]
    fn next_version_some_increments() {
        assert_eq!(next_version(Some(3)), 4);
        assert_eq!(next_version(Some(1)), 2);
        assert_eq!(next_version(Some(0)), 1);
    }

    #[test]
    fn snapshot_roundtrips_serde() {
        let snap = sample_snapshot();
        let s = serde_json::to_string(&snap).unwrap();
        let back: AgentSnapshot = serde_json::from_str(&s).unwrap();
        assert_eq!(snap, back);
    }
}
