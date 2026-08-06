//! Spawn Preset 路由：列表 / 创建 / 查询 / 更新 / 删除。

use axum::Json;
use axum::extract::{Path, State};
use lol_web_protocol::spawn_preset::{CreateSpawnPresetDto, SpawnPreset};
use uuid::Uuid;

use super::response::{ApiResponse, api_error};
use super::{AppState, AuthUser};

pub async fn list_spawn_presets(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<SpawnPreset>> {
    match s.spawn_preset_service.list(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list.into_iter().map(Into::into).collect()),
        Err(e) => api_error(e),
    }
}

pub async fn create_spawn_preset(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(input): Json<CreateSpawnPresetDto>,
) -> ApiResponse<SpawnPreset> {
    let domain_input = input.into();
    match s
        .spawn_preset_service
        .create(auth.user_id, domain_input)
        .await
    {
        Ok(p) => ApiResponse::ok(p.into()),
        Err(e) => api_error(e),
    }
}

pub async fn get_spawn_preset(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<SpawnPreset> {
    match s.spawn_preset_service.get(auth.user_id, id).await {
        Ok(p) => ApiResponse::ok(p.into()),
        Err(e) => api_error(e),
    }
}

pub async fn update_spawn_preset(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateSpawnPresetDto>,
) -> ApiResponse<()> {
    let domain_input = input.into();
    match s
        .spawn_preset_service
        .update(auth.user_id, id, domain_input)
        .await
    {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}

pub async fn delete_spawn_preset(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.spawn_preset_service.delete(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}
