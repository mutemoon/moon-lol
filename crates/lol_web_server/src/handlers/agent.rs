//! Agent 路由：列表 / 创建 / 查询 / 更新 / 删除 / 可见性。

use axum::Json;
use axum::extract::{Path, State};
use lol_web_protocol::agent::{Agent, CreateAgentDto};
use serde::Deserialize;
use uuid::Uuid;

use super::response::{ApiResponse, api_error};
use super::{AppState, AuthUser};

pub async fn list_agents(auth: AuthUser, State(s): State<AppState>) -> ApiResponse<Vec<Agent>> {
    match s.agent_service.list(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list.into_iter().map(Into::into).collect()),
        Err(e) => api_error(e),
    }
}

pub async fn create_agent(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(input): Json<CreateAgentDto>,
) -> ApiResponse<Agent> {
    let domain_input = input.into();
    match s.agent_service.create(auth.user_id, domain_input).await {
        Ok(a) => ApiResponse::ok(a.into()),
        Err(e) => api_error(e),
    }
}

pub async fn get_agent(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Agent> {
    match s.agent_service.get(auth.user_id, id).await {
        Ok(a) => ApiResponse::ok(a.into()),
        Err(e) => api_error(e),
    }
}

pub async fn update_agent(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateAgentDto>,
) -> ApiResponse<()> {
    let domain_input = input.into();
    match s.agent_service.update(auth.user_id, id, domain_input).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}

pub async fn delete_agent(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.agent_service.delete(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}

#[derive(Deserialize)]
pub struct UpdateVisibilityRequest {
    pub visibility: lol_web_protocol::Visibility,
}

pub async fn update_agent_visibility(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateVisibilityRequest>,
) -> ApiResponse<()> {
    let domain_visibility: crate::domain::spawn_preset::Visibility = req.visibility.into();
    match s
        .agent_service
        .update_visibility(auth.user_id, id, domain_visibility)
        .await
    {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}
