//! Admin 路由：指标 / 运行中对局 / 强制中止。

use axum::extract::{Path, State};
use lol_web_protocol::admin::AdminMetrics;
use uuid::Uuid;

use super::response::{ApiResponse, api_error};
use super::{AppState, AuthUser};

pub async fn admin_metrics(
    _auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<AdminMetrics> {
    match s.admin_service.metrics().await {
        Ok(m) => ApiResponse::ok(m.into()),
        Err(e) => api_error(e),
    }
}

pub async fn admin_running(
    _auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<lol_web_protocol::match_::Match>> {
    match s.admin_service.list_running().await {
        Ok(list) => ApiResponse::ok(list.into_iter().map(Into::into).collect()),
        Err(e) => api_error(e),
    }
}

pub async fn admin_force_abort(
    _auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.admin_service.force_abort(id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}
