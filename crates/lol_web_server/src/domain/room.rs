//! Room 子系统的领域层（房间 + 成员 + Agent 槽位 + 房主约束）。

use serde::{Deserialize, Serialize};

use super::spawn_preset::Team;

/// 房间状态。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RoomStatus {
    Lobby,
    Running,
    Closed,
}

impl RoomStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoomStatus::Lobby => "lobby",
            RoomStatus::Running => "running",
            RoomStatus::Closed => "closed",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "lobby" => Some(RoomStatus::Lobby),
            "running" => Some(RoomStatus::Running),
            "closed" => Some(RoomStatus::Closed),
            _ => None,
        }
    }
}

/// 阵营策略。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TeamPolicy {
    /// 单阵营：每人只能在 Order 或 Chaos 一方。
    SingleTeam,
    /// 自由：不限制阵营。
    Free,
}

impl TeamPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            TeamPolicy::SingleTeam => "single_team",
            TeamPolicy::Free => "free",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "single_team" => Some(TeamPolicy::SingleTeam),
            "free" => Some(TeamPolicy::Free),
            _ => None,
        }
    }
}

/// 房间约束（房主创建时设定）。
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RoomConstraints {
    pub max_members: i32,
    pub max_agents_per_member: i32,
    pub team_policy: TeamPolicy,
    pub lobby_visible: bool,
    pub prompt_visible: bool,
}

impl Default for RoomConstraints {
    fn default() -> Self {
        Self {
            max_members: 10,
            max_agents_per_member: 3,
            team_policy: TeamPolicy::Free,
            lobby_visible: true,
            prompt_visible: false,
        }
    }
}

/// 房间领域类型。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Room {
    pub id: uuid::Uuid,
    pub owner_id: i32,
    pub name: String,
    pub invite_code: String,
    pub constraints: RoomConstraints,
    pub status: RoomStatus,
}

/// 房间内 Agent 槽位。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RoomAgentSlot {
    pub id: uuid::Uuid,
    pub room_id: uuid::Uuid,
    pub user_id: i32,
    pub agent_id: uuid::Uuid,
    pub team: Team,
}

/// 校验添加槽位是否合规（纯函数，无 IO）。
///
/// 规则（PRODUCT.md §3.B.3 / §3.B.4）：
/// - 房间必须处于 lobby 状态
/// - 成员已有槽位数 < max_agents_per_member
/// - 若 team_policy=SingleTeam，该成员已有槽位的 team 必须一致（或尚无槽位）
pub fn validate_add_slot(
    room_status: RoomStatus,
    constraints: RoomConstraints,
    member_current_slots: i32,
    member_existing_team: Option<Team>,
    new_team: Team,
) -> Result<(), RoomValidationError> {
    use RoomValidationError::*;
    if room_status != RoomStatus::Lobby {
        return Err(NotInLobby);
    }
    if member_current_slots >= constraints.max_agents_per_member {
        return Err(AgentLimitExceeded {
            current: member_current_slots,
            limit: constraints.max_agents_per_member,
        });
    }
    if constraints.team_policy == TeamPolicy::SingleTeam {
        if let Some(existing) = member_existing_team {
            if existing != new_team {
                return Err(TeamPolicyViolation {
                    existing,
                    requested: new_team,
                });
            }
        }
    }
    Ok(())
}

/// 校验加入房间是否合规。
pub fn validate_join(
    room_status: RoomStatus,
    constraints: RoomConstraints,
    current_members: i32,
) -> Result<(), RoomValidationError> {
    if room_status != RoomStatus::Lobby {
        return Err(RoomValidationError::NotInLobby);
    }
    if current_members >= constraints.max_members {
        return Err(RoomValidationError::MemberLimitExceeded {
            current: current_members,
            limit: constraints.max_members,
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoomValidationError {
    NotInLobby,
    AgentLimitExceeded { current: i32, limit: i32 },
    MemberLimitExceeded { current: i32, limit: i32 },
    TeamPolicyViolation { existing: Team, requested: Team },
}

pub fn validate_room_name(name: &str) -> bool {
    let n = name.trim();
    !n.is_empty() && n.len() <= 64
}

/// 生成 6 位邀请码（大写字母+数字）。
pub fn generate_invite_code() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::rng();
    (0..6)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_constraints() -> RoomConstraints {
        RoomConstraints::default()
    }

    #[test]
    fn validate_add_slot_ok_when_below_limit() {
        let result = validate_add_slot(
            RoomStatus::Lobby,
            default_constraints(),
            1,
            None,
            Team::Order,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn validate_add_slot_rejected_when_not_lobby() {
        let result = validate_add_slot(
            RoomStatus::Running,
            default_constraints(),
            0,
            None,
            Team::Order,
        );
        assert_eq!(result.unwrap_err(), RoomValidationError::NotInLobby);
    }

    #[test]
    fn validate_add_slot_rejected_at_agent_limit() {
        let result = validate_add_slot(
            RoomStatus::Lobby,
            default_constraints(),
            3, // = max_agents_per_member
            Some(Team::Order),
            Team::Order,
        );
        assert_eq!(
            result.unwrap_err(),
            RoomValidationError::AgentLimitExceeded {
                current: 3,
                limit: 3
            }
        );
    }

    #[test]
    fn validate_add_slot_single_team_policy_allows_same_team() {
        let mut c = default_constraints();
        c.team_policy = TeamPolicy::SingleTeam;
        let result = validate_add_slot(RoomStatus::Lobby, c, 1, Some(Team::Order), Team::Order);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_add_slot_single_team_policy_rejects_different_team() {
        let mut c = default_constraints();
        c.team_policy = TeamPolicy::SingleTeam;
        let result = validate_add_slot(RoomStatus::Lobby, c, 1, Some(Team::Order), Team::Chaos);
        assert_eq!(
            result.unwrap_err(),
            RoomValidationError::TeamPolicyViolation {
                existing: Team::Order,
                requested: Team::Chaos,
            }
        );
    }

    #[test]
    fn validate_add_slot_free_policy_allows_different_teams() {
        let result = validate_add_slot(
            RoomStatus::Lobby,
            default_constraints(),
            1,
            Some(Team::Order),
            Team::Chaos,
        );
        assert!(result.is_ok(), "free 策略允许跨阵营");
    }

    #[test]
    fn validate_join_ok_below_member_limit() {
        let result = validate_join(RoomStatus::Lobby, default_constraints(), 5);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_join_rejected_at_member_limit() {
        let result = validate_join(RoomStatus::Lobby, default_constraints(), 10);
        assert_eq!(
            result.unwrap_err(),
            RoomValidationError::MemberLimitExceeded {
                current: 10,
                limit: 10
            }
        );
    }

    #[test]
    fn validate_join_rejected_when_running() {
        let result = validate_join(RoomStatus::Running, default_constraints(), 5);
        assert_eq!(result.unwrap_err(), RoomValidationError::NotInLobby);
    }

    #[test]
    fn invite_code_is_6_chars() {
        let code = generate_invite_code();
        assert_eq!(code.len(), 6);
        assert!(
            code.chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        );
    }

    #[test]
    fn room_name_validation() {
        assert!(validate_room_name("锐雯内战房"));
        assert!(!validate_room_name(""));
        assert!(!validate_room_name(&"x".repeat(65)));
    }

    #[test]
    fn status_roundtrip() {
        assert_eq!(RoomStatus::from_str("lobby"), Some(RoomStatus::Lobby));
        assert_eq!(RoomStatus::from_str("running"), Some(RoomStatus::Running));
        assert_eq!(RoomStatus::from_str("x"), None);
    }

    #[test]
    fn team_policy_roundtrip() {
        assert_eq!(TeamPolicy::from_str("free"), Some(TeamPolicy::Free));
        assert_eq!(
            TeamPolicy::from_str("single_team"),
            Some(TeamPolicy::SingleTeam)
        );
        assert_eq!(TeamPolicy::from_str("x"), None);
    }
}
