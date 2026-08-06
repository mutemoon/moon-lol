//! Rank wire DTO（ELO 评分、赛季、排队）。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankQueueEntry {
    pub user_id: i32,
    pub agent_id: Uuid,
    pub agent_snapshot_id: Uuid,
    pub mode: String,
    pub rating: f64,
    pub enqueued_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EloRating {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub mode: String,
    pub rating: f64,
    pub games_played: i32,
    pub wins: i32,
    pub losses: i32,
    pub daily_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    pub id: Uuid,
    pub mode: String,
    pub starts_at: String,
    pub ends_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankEnqueueRequest {
    pub agent_id: Uuid,
    pub agent_snapshot_id: Uuid,
    pub mode: String,
}
