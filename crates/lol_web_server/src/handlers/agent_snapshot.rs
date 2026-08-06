//! Agent Snapshot 路由：发布快照 / 列出快照。

use axum::extract::{Path, State};
use lol_web_protocol::agent_snapshot::AgentSnapshot;
use uuid::Uuid;

use super::response::{ApiResponse, api_error};
use super::{AppState, AuthUser};
use crate::service::agent_snapshot_service::build_config_freeze;

pub async fn publish_snapshot(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<AgentSnapshot> {
    let agent = match s.agent_service.get(auth.user_id, id).await {
        Ok(a) => a,
        Err(e) => return api_error(e),
    };
    let freeze = build_config_freeze(&agent, None, None);
    match s
        .agent_snapshot_service
        .publish(auth.user_id, id, freeze)
        .await
    {
        Ok(snap) => ApiResponse::ok(snap.into()),
        Err(e) => api_error(e),
    }
}

pub async fn list_snapshots(
    _auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Vec<AgentSnapshot>> {
    match s.agent_snapshot_service.list_by_agent(id).await {
        Ok(list) => ApiResponse::ok(list.into_iter().map(Into::into).collect()),
        Err(e) => api_error(e),
    }
}
