//! Model Provider 路由：列表 / 创建 / 更新 / 删除。

use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use super::response::ApiResponse;
use super::{AppState, AuthUser};

pub async fn list_model_providers(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<crate::domain::model_provider::ModelProviderDto>> {
    match s.model_provider_service.list(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn create_model_provider(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(input): Json<crate::domain::model_provider::ModelProviderInput>,
) -> ApiResponse<crate::domain::model_provider::ModelProviderDto> {
    match s.model_provider_service.create(auth.user_id, input).await {
        Ok(dto) => ApiResponse::ok(dto),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn update_model_provider(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<crate::domain::model_provider::ModelProviderInput>,
) -> ApiResponse<()> {
    match s
        .model_provider_service
        .update(auth.user_id, id, input)
        .await
    {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn delete_model_provider(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.model_provider_service.delete(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}
