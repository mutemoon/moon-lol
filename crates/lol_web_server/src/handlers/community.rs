//! Community 路由：浏览公开 Agent / fork / pull-upstream。

use axum::Json;
use axum::extract::{Path, Query, State};
use lol_web_protocol::agent::Agent;
use serde::Deserialize;
use uuid::Uuid;

use super::response::{ApiResponse, api_error};
use super::{AppState, AuthUser};

#[derive(Deserialize)]
pub struct BrowseQuery {
    pub sort: Option<String>,
    pub limit: Option<i64>,
}

pub async fn browse_community(
    _auth: AuthUser,
    State(s): State<AppState>,
    Query(q): Query<BrowseQuery>,
) -> ApiResponse<Vec<Agent>> {
    let sort = q
        .sort
        .as_deref()
        .and_then(crate::domain::community::CommunitySort::from_str)
        .unwrap_or(crate::domain::community::CommunitySort::Recent);
    match s
        .community_service
        .browse_public(sort, q.limit.unwrap_or(50))
        .await
    {
        Ok(list) => ApiResponse::ok(list.into_iter().map(Into::into).collect()),
        Err(e) => api_error(e),
    }
}

#[derive(Deserialize)]
pub struct ForkRequest {
    pub new_name: Option<String>,
}

pub async fn fork_agent(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<ForkRequest>,
) -> ApiResponse<Agent> {
    match s
        .community_service
        .fork(auth.user_id, id, req.new_name)
        .await
    {
        Ok(a) => ApiResponse::ok(a.into()),
        Err(e) => api_error(e),
    }
}

pub async fn pull_upstream_agent(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Agent> {
    match s.community_service.pull_upstream(auth.user_id, id).await {
        Ok(a) => ApiResponse::ok(a.into()),
        Err(e) => api_error(e),
    }
}
