//! Room wire DTO（房间 + 成员槽位）。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::spawn_preset::Team;

// ── 房间约束 ──

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct RoomConstraints {
    pub max_members: i32,
    pub max_agents_per_member: i32,
    #[serde(rename = "team_policy")]
    pub team_policy: TeamPolicy,
    pub lobby_visible: bool,
    pub prompt_visible: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TeamPolicy {
    SingleTeam,
    Free,
}

// ── 房间状态 ──

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RoomStatus {
    Lobby,
    Running,
    Closed,
}

// ── Room DTO ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub owner_id: i32,
    pub constraints: RoomConstraints,
    pub invite_code: String,
    #[serde(default)]
    pub member_count: Option<i32>,
    pub status: RoomStatus,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoomAgentSlot {
    pub id: Uuid,
    pub room_id: Uuid,
    pub member_user_id: i32,
    pub agent_id: Uuid,
    pub team: Team,
}

// ── 请求 DTO ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub constraints: RoomConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSlotRequest {
    pub agent_id: Uuid,
    pub team: Team,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinByCodeRequest {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRoomResponse {
    pub match_id: Uuid,
    pub ws_port: i32,
}
