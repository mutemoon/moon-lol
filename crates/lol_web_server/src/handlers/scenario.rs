//! Scenario 路由：列表 / 创建 / 查询 / 更新 / 删除 / 胜负条件。

use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use super::response::ApiResponse;
use super::{AppState, AuthUser};

pub async fn list_scenarios(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<crate::domain::scenario::Scenario>> {
    match s.scenario_service.list(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn create_scenario(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(input): Json<crate::domain::scenario::ScenarioInput>,
) -> ApiResponse<crate::domain::scenario::Scenario> {
    match s.scenario_service.create(auth.user_id, input).await {
        Ok(sc) => ApiResponse::ok(sc),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn get_scenario(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<crate::domain::scenario::Scenario> {
    match s.scenario_service.get(auth.user_id, id).await {
        Ok(sc) => ApiResponse::ok(sc),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn update_scenario(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<crate::domain::scenario::ScenarioInput>,
) -> ApiResponse<()> {
    match s.scenario_service.update(auth.user_id, id, input).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn delete_scenario(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.scenario_service.delete(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn get_win_condition(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Option<serde_json::Value>> {
    match s.scenario_service.get_win_condition(auth.user_id, id).await {
        Ok(wc) => ApiResponse::ok(wc),
        Err(e) => ApiResponse::from_error(e),
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
        Err(e) => ApiResponse::from_error(e),
    }
}
