//! Match wire DTO（对局实例、事件、胜负）。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── 对局状态 ──

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MatchStatus {
    Pending,
    Running,
    Paused,
    Finished,
    Aborted,
}

// ── 胜方 ──

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Winner {
    Order,
    Chaos,
    None,
}

// ── Match DTO ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Match {
    pub id: Uuid,
    pub mode: String,
    pub status: MatchStatus,
    pub owner_user_id: Option<i32>,
    pub room_id: Option<Uuid>,
    pub ws_port: Option<i32>,
    pub created_at: String,
    pub finished_at: Option<String>,
}

// ── 对局事件 ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatchEvent {
    /// DB 自增 id（BIGSERIAL），JSON 序列化为字符串与前端 string 契约一致
    pub id: i64,
    pub match_id: Uuid,
    pub seq: i32,
    pub payload: serde_json::Value,
    pub recorded_at: String,
}

// ── roundtrip 单测 ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_status_serializes_lowercase() {
        assert_eq!(
            serde_json::to_string(&MatchStatus::Pending).unwrap(),
            r#""pending""#
        );
        assert_eq!(
            serde_json::to_string(&MatchStatus::Running).unwrap(),
            r#""running""#
        );
        assert_eq!(
            serde_json::to_string(&MatchStatus::Paused).unwrap(),
            r#""paused""#
        );
        assert_eq!(
            serde_json::to_string(&MatchStatus::Finished).unwrap(),
            r#""finished""#
        );
        assert_eq!(
            serde_json::to_string(&MatchStatus::Aborted).unwrap(),
            r#""aborted""#
        );
    }

    #[test]
    fn match_status_roundtrip() {
        let cases = ["pending", "running", "paused", "finished", "aborted"];
        for s in cases {
            let ms: MatchStatus = serde_json::from_str(&format!(r#""{s}""#)).unwrap();
            assert_eq!(serde_json::to_string(&ms).unwrap(), format!(r#""{s}""#));
        }
    }

    #[test]
    fn winner_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&Winner::Order).unwrap(), r#""order""#);
        assert_eq!(serde_json::to_string(&Winner::Chaos).unwrap(), r#""chaos""#);
        assert_eq!(serde_json::to_string(&Winner::None).unwrap(), r#""none""#);
    }

    #[test]
    fn winner_roundtrip() {
        let cases = ["order", "chaos", "none"];
        for s in cases {
            let w: Winner = serde_json::from_str(&format!(r#""{s}""#)).unwrap();
            assert_eq!(serde_json::to_string(&w).unwrap(), format!(r#""{s}""#));
        }
    }
}
