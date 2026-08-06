//! Scenario 路由：列表 / 创建 / 查询 / 更新 / 删除 / 胜负条件。

use axum::Json;
use axum::extract::{Path, State};
use lol_web_protocol::scenario::{CreateScenarioDto, Scenario};
use uuid::Uuid;

use super::response::{ApiResponse, api_error};
use super::{AppState, AuthUser};

pub async fn list_scenarios(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<Scenario>> {
    match s.scenario_service.list(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list.into_iter().map(Into::into).collect()),
        Err(e) => api_error(e),
    }
}

pub async fn create_scenario(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(input): Json<CreateScenarioDto>,
) -> ApiResponse<Scenario> {
    let domain_input = input.into();
    match s.scenario_service.create(auth.user_id, domain_input).await {
        Ok(sc) => ApiResponse::ok(sc.into()),
        Err(e) => api_error(e),
    }
}

pub async fn get_scenario(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Scenario> {
    match s.scenario_service.get(auth.user_id, id).await {
        Ok(sc) => ApiResponse::ok(sc.into()),
        Err(e) => api_error(e),
    }
}

pub async fn update_scenario(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateScenarioDto>,
) -> ApiResponse<()> {
    let domain_input = input.into();
    match s
        .scenario_service
        .update(auth.user_id, id, domain_input)
        .await
    {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}

pub async fn delete_scenario(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.scenario_service.delete(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}

pub async fn get_win_condition(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Option<serde_json::Value>> {
    match s.scenario_service.get_win_condition(auth.user_id, id).await {
        Ok(wc) => ApiResponse::ok(wc),
        Err(e) => api_error(e),
    }
}

pub async fn save_win_condition(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(cond): Json<serde_json::Value>,
) -> ApiResponse<()> {
    match s
        .scenario_service
        .save_win_condition(auth.user_id, id, cond)
        .await
    {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}
