//! Match 子系统的领域层（对局实例，统一三形态）。

use serde::{Deserialize, Serialize};

use super::spawn_preset::Team;

/// 对局形态。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MatchForm {
    Local,
    Room,
    Rank,
}

impl MatchForm {
    pub fn as_str(&self) -> &'static str {
        match self {
            MatchForm::Local => "local",
            MatchForm::Room => "room",
            MatchForm::Rank => "rank",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "local" => Some(MatchForm::Local),
            "room" => Some(MatchForm::Room),
            "rank" => Some(MatchForm::Rank),
            _ => None,
        }
    }
}

/// 对局状态。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MatchStatus {
    Pending,
    Running,
    Paused,
    Finished,
    Aborted,
}

impl MatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MatchStatus::Pending => "pending",
            MatchStatus::Running => "running",
            MatchStatus::Paused => "paused",
            MatchStatus::Finished => "finished",
            MatchStatus::Aborted => "aborted",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(MatchStatus::Pending),
            "running" => Some(MatchStatus::Running),
            "paused" => Some(MatchStatus::Paused),
            "finished" => Some(MatchStatus::Finished),
            "aborted" => Some(MatchStatus::Aborted),
            _ => None,
        }
    }
}

/// 对局结果。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParticipantResult {
    Win,
    Loss,
    Draw,
    None,
}

impl ParticipantResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            ParticipantResult::Win => "win",
            ParticipantResult::Loss => "loss",
            ParticipantResult::Draw => "draw",
            ParticipantResult::None => "none",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "win" => Some(ParticipantResult::Win),
            "loss" => Some(ParticipantResult::Loss),
            "draw" => Some(ParticipantResult::Draw),
            "none" => Some(ParticipantResult::None),
            _ => None,
        }
    }
}

/// 胜方（none 表示中止无胜负）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Winner {
    Order,
    Chaos,
    None,
}

impl Winner {
    pub fn as_str(&self) -> &'static str {
        match self {
            Winner::Order => "order",
            Winner::Chaos => "chaos",
            Winner::None => "none",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "order" => Some(Winner::Order),
            "chaos" => Some(Winner::Chaos),
            "none" => Some(Winner::None),
            _ => None,
        }
    }

    /// 根据胜方判定某个 team 的结果。
    pub fn result_for_team(&self, team: Team) -> ParticipantResult {
        match (self, team) {
            (Winner::None, _) => ParticipantResult::None,
            (Winner::Order, Team::Order) | (Winner::Chaos, Team::Chaos) => ParticipantResult::Win,
            _ => ParticipantResult::Loss,
        }
    }
}

/// 对局实例领域类型。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Match {
    pub id: uuid::Uuid,
    pub form: MatchForm,
    pub room_id: Option<uuid::Uuid>,
    pub owner_id: i32,
    pub mode: String,
    pub status: MatchStatus,
    pub bevy_port: Option<i32>,
    pub winner_team: Option<Winner>,
    pub abort_reason: Option<String>,
}

/// 对局参与者。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatchParticipant {
    pub id: uuid::Uuid,
    pub match_id: uuid::Uuid,
    pub agent_snapshot_id: uuid::Uuid,
    pub agent_id: uuid::Uuid,
    pub user_id: i32,
    pub team: Team,
    pub result: Option<ParticipantResult>,
}

/// 对局事件（操作流）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatchEvent {
    pub seq: i32,
    pub event_type: String,
    pub agent_id: Option<uuid::Uuid>,
    pub payload: serde_json::Value,
    pub game_time_ms: i64,
}

/// 状态机：判定状态转换是否合法。
pub fn can_transition(from: MatchStatus, to: MatchStatus) -> bool {
    use MatchStatus::*;
    matches!(
        (from, to),
        (Pending, Running)
            | (Running, Paused)
            | (Paused, Running)
            | (Running, Finished)
            | (Running, Aborted)
            | (Paused, Aborted)
            | (Pending, Aborted)
    )
}

/// 默认 Bevy 端口池范围。
pub const PORT_POOL_START: i32 = 9100;
pub const PORT_POOL_END: i32 = 9200;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_roundtrip() {
        assert_eq!(MatchForm::from_str("local"), Some(MatchForm::Local));
        assert_eq!(MatchForm::from_str("rank"), Some(MatchForm::Rank));
        assert_eq!(MatchForm::from_str("x"), None);
    }

    #[test]
    fn status_roundtrip() {
        assert_eq!(MatchStatus::from_str("running"), Some(MatchStatus::Running));
        assert_eq!(MatchStatus::from_str("aborted"), Some(MatchStatus::Aborted));
        assert_eq!(MatchStatus::from_str("x"), None);
    }

    #[test]
    fn winner_result_for_team() {
        assert_eq!(
            Winner::Order.result_for_team(Team::Order),
            ParticipantResult::Win
        );
        assert_eq!(
            Winner::Order.result_for_team(Team::Chaos),
            ParticipantResult::Loss
        );
        assert_eq!(
            Winner::Chaos.result_for_team(Team::Chaos),
            ParticipantResult::Win
        );
        assert_eq!(
            Winner::None.result_for_team(Team::Order),
            ParticipantResult::None
        );
    }

    #[test]
    fn transition_pending_to_running_allowed() {
        assert!(can_transition(MatchStatus::Pending, MatchStatus::Running));
    }

    #[test]
    fn transition_running_to_paused_allowed() {
        assert!(can_transition(MatchStatus::Running, MatchStatus::Paused));
    }

    #[test]
    fn transition_paused_to_running_allowed() {
        assert!(can_transition(MatchStatus::Paused, MatchStatus::Running));
    }

    #[test]
    fn transition_running_to_finished_allowed() {
        assert!(can_transition(MatchStatus::Running, MatchStatus::Finished));
    }

    #[test]
    fn transition_finished_to_running_forbidden() {
        assert!(!can_transition(MatchStatus::Finished, MatchStatus::Running));
    }

    #[test]
    fn transition_finished_to_anything_forbidden() {
        assert!(!can_transition(MatchStatus::Finished, MatchStatus::Aborted));
        assert!(!can_transition(MatchStatus::Finished, MatchStatus::Paused));
    }

    #[test]
    fn transition_aborted_is_terminal() {
        assert!(!can_transition(MatchStatus::Aborted, MatchStatus::Running));
    }

    #[test]
    fn transition_running_to_aborted_allowed() {
        assert!(can_transition(MatchStatus::Running, MatchStatus::Aborted));
    }
}
