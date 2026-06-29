//! Agent 路由：列表 / 创建 / 查询 / 更新 / 删除 / 可见性。

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use super::response::ApiResponse;
use super::{AppState, AuthUser};

pub async fn list_agents(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<crate::domain::agent::Agent>> {
    match s.agent_service.list(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn create_agent(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(input): Json<crate::domain::agent::AgentInput>,
) -> ApiResponse<crate::domain::agent::Agent> {
    match s.agent_service.create(auth.user_id, input).await {
        Ok(a) => ApiResponse::ok(a),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn get_agent(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<crate::domain::agent::Agent> {
    match s.agent_service.get(auth.user_id, id).await {
        Ok(a) => ApiResponse::ok(a),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn update_agent(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<crate::domain::agent::AgentInput>,
) -> ApiResponse<()> {
    match s.agent_service.update(auth.user_id, id, input).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn delete_agent(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.agent_service.delete(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Deserialize)]
pub struct UpdateVisibilityRequest {
    pub visibility: crate::domain::spawn_preset::Visibility,
}

pub async fn update_agent_visibility(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateVisibilityRequest>,
) -> ApiResponse<()> {
    match s
        .agent_service
        .update_visibility(auth.user_id, id, req.visibility)
        .await
    {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}
