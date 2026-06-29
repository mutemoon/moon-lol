//! Match 路由：列表 / 查询 / 事件流 / 停止对局。

use axum::extract::{Path, Query, State};
use serde::Deserialize;
use uuid::Uuid;

use super::response::ApiResponse;
use super::{AppState, AuthUser};

#[derive(Deserialize)]
pub struct ListMatchesQuery {
    pub status: Option<String>,
}

pub async fn list_matches(
    auth: AuthUser,
    State(s): State<AppState>,
    Query(q): Query<ListMatchesQuery>,
) -> ApiResponse<Vec<crate::domain::match_::Match>> {
    if let Some(status_str) = q.status {
        if let Some(status) = crate::domain::match_::MatchStatus::from_str(&status_str) {
            return match s.match_service.list_by_status(status).await {
                Ok(list) => ApiResponse::ok(list),
                Err(e) => ApiResponse::from_error(e),
            };
        }
    }
    match s.match_service.list_mine(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn get_match(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<crate::domain::match_::Match> {
    match s.match_service.get(auth.user_id, id).await {
        Ok(m) => ApiResponse::ok(m),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Deserialize)]
pub struct GetEventsQuery {
    pub from_seq: Option<i32>,
    pub limit: Option<i64>,
}

pub async fn get_match_events(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<GetEventsQuery>,
) -> ApiResponse<Vec<crate::domain::match_::MatchEvent>> {
    match s
        .match_service
        .get_events(
            auth.user_id,
            id,
            q.from_seq.unwrap_or(0),
            q.limit.unwrap_or(100),
        )
        .await
    {
        Ok(events) => ApiResponse::ok(events),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn stop_match(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.local_game_service.stop(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}
