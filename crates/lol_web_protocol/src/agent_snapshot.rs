//! AgentSnapshot wire DTO（参赛快照，Rank 队列用）。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentSnapshot {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub version: i32,
    pub config_freeze: serde_json::Value,
    pub created_at: String,
}
