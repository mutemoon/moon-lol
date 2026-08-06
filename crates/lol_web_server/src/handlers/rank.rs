//! Rank 路由：排队 / 队列状态 / 排行榜 / 当前赛季。

use axum::Json;
use axum::extract::{Query, State};
use lol_web_protocol::rank::{EloRating, RankEnqueueRequest, RankQueueEntry, Season};
use serde::Deserialize;

use super::response::{ApiResponse, api_error};
use super::{AppState, AuthUser};

pub async fn rank_enqueue(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(req): Json<RankEnqueueRequest>,
) -> ApiResponse<RankQueueEntry> {
    match s
        .rank_service
        .enqueue(auth.user_id, req.agent_id, req.agent_snapshot_id, &req.mode)
        .await
    {
        Ok(entry) => ApiResponse::ok(entry.into()),
        Err(e) => api_error(e),
    }
}

pub async fn rank_queue_status(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<RankQueueEntry>> {
    match s.rank_service.list_my_queue(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list.into_iter().map(Into::into).collect()),
        Err(e) => api_error(e),
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
) -> ApiResponse<Vec<EloRating>> {
    let mode = q.mode.as_deref().unwrap_or("top_solo");
    match s
        .rank_service
        .leaderboard(mode, q.limit.unwrap_or(50))
        .await
    {
        Ok(list) => ApiResponse::ok(list.into_iter().map(Into::into).collect()),
        Err(e) => api_error(e),
    }
}

pub async fn current_season(_auth: AuthUser, State(s): State<AppState>) -> ApiResponse<Season> {
    match s.rank_service.current_season("top_solo").await {
        Ok(season) => ApiResponse::ok(season.into()),
        Err(e) => api_error(e),
    }
}
