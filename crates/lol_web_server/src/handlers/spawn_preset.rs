//! Spawn Preset 路由：列表 / 创建 / 查询 / 更新 / 删除。

use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use super::response::ApiResponse;
use super::{AppState, AuthUser};

pub async fn list_spawn_presets(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<crate::domain::spawn_preset::SpawnPreset>> {
    match s.spawn_preset_service.list(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn create_spawn_preset(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(input): Json<crate::domain::spawn_preset::SpawnPresetInput>,
) -> ApiResponse<crate::domain::spawn_preset::SpawnPreset> {
    match s.spawn_preset_service.create(auth.user_id, input).await {
        Ok(p) => ApiResponse::ok(p),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn get_spawn_preset(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<crate::domain::spawn_preset::SpawnPreset> {
    match s.spawn_preset_service.get(auth.user_id, id).await {
        Ok(p) => ApiResponse::ok(p),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn update_spawn_preset(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<crate::domain::spawn_preset::SpawnPresetInput>,
) -> ApiResponse<()> {
    match s.spawn_preset_service.update(auth.user_id, id, input).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn delete_spawn_preset(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.spawn_preset_service.delete(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}
