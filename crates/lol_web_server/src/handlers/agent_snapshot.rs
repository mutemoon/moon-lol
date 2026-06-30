//! Agent Snapshot 路由：发布快照 / 列出快照。

use axum::extract::{Path, State};
use uuid::Uuid;

use super::response::ApiResponse;
use super::{AppState, AuthUser};
use crate::service::agent_snapshot_service::build_config_freeze;

pub async fn publish_snapshot(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<crate::domain::agent_snapshot::AgentSnapshot> {
    // 冻结当前 Agent 配置（含 model 与 config_json.provider_id）作为参赛快照。
    let agent = match s.agent_service.get(auth.user_id, id).await {
        Ok(a) => a,
        Err(e) => return ApiResponse::from_error(e),
    };
    let freeze = build_config_freeze(&agent, None, None);
    match s
        .agent_snapshot_service
        .publish(auth.user_id, id, freeze)
        .await
    {
        Ok(snap) => ApiResponse::ok(snap),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn list_snapshots(
    _auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Vec<crate::domain::agent_snapshot::AgentSnapshot>> {
    match s.agent_snapshot_service.list_by_agent(id).await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}
