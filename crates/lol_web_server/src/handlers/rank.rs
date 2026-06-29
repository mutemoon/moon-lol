//! Rank 路由：排队 / 队列状态 / 排行榜 / 当前赛季。

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use uuid::Uuid;

use super::response::ApiResponse;
use super::{AppState, AuthUser};

#[derive(Deserialize)]
pub struct RankEnqueueRequest {
    pub agent_id: Uuid,
    pub agent_snapshot_id: Uuid,
    pub mode: String,
}

pub async fn rank_enqueue(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(req): Json<RankEnqueueRequest>,
) -> ApiResponse<crate::repository::rank_repo::RankQueueEntry> {
    match s
        .rank_service
        .enqueue(auth.user_id, req.agent_id, req.agent_snapshot_id, &req.mode)
        .await
    {
        Ok(entry) => ApiResponse::ok(entry),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn rank_queue_status(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<crate::repository::rank_repo::RankQueueEntry>> {
    match s.rank_service.list_my_queue(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Deserialize)]
pub struct LeaderboardQuery {
    pub mode: Option<String>,
    pub limit: Option<i64>,
}

pub async fn rank_leaderboard(
    _auth: AuthUser,
    State(s): State<AppState>,
    Query(q): Query<LeaderboardQuery>,
) -> ApiResponse<Vec<crate::repository::rank_repo::EloRating>> {
    let mode = q.mode.as_deref().unwrap_or("top_solo");
    match s
        .rank_service
        .leaderboard(mode, q.limit.unwrap_or(50))
        .await
    {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn current_season(
    _auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<crate::repository::rank_repo::Season> {
    match s.rank_service.current_season("top_solo").await {
        Ok(season) => ApiResponse::ok(season),
        Err(e) => ApiResponse::from_error(e),
    }
}
