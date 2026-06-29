//! Agent Snapshot 路由：发布快照 / 列出快照。

use axum::extract::{Path, State};
use uuid::Uuid;

use super::response::ApiResponse;
use super::{AppState, AuthUser};

pub async fn publish_snapshot(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<crate::domain::agent_snapshot::AgentSnapshot> {
    // 简化：config_freeze 传空对象，实际应由前端构造
    match s
        .agent_snapshot_service
        .publish(auth.user_id, id, serde_json::json!({}))
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
