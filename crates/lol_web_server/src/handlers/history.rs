//! 游戏历史 handler：列表 / 详情 / 上传 / 删除。

use axum::Json;
use axum::extract::{Path, State};
use lol_web_protocol::history::{GameHistorySummary, SavedAgentHistory, UploadHistoryRequest};
use uuid::Uuid;

use super::response::{ApiResponse, api_error};
use super::{AppState, AuthUser};

pub async fn list_histories(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<GameHistorySummary>> {
    match s.history_service.list(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => api_error(e),
    }
}

pub async fn get_history(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Vec<SavedAgentHistory>> {
    match s.history_service.get(auth.user_id, id).await {
        Ok(histories) => ApiResponse::ok(histories),
        Err(e) => api_error(e),
    }
}

pub async fn upload_history(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(req): Json<UploadHistoryRequest>,
) -> ApiResponse<()> {
    match s.history_service.upload(auth.user_id, req).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}

pub async fn delete_history(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.history_service.delete(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}
