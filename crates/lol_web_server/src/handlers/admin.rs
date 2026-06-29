//! Admin 路由：指标 / 运行中对局 / 强制中止。

use axum::extract::{Path, State};
use uuid::Uuid;

use super::response::ApiResponse;
use super::AppState;
use super::AuthUser;
use crate::service::AdminMetrics;

pub async fn admin_metrics(_auth: AuthUser, State(s): State<AppState>) -> ApiResponse<AdminMetrics> {
    match s.admin_service.metrics().await {
        Ok(m) => ApiResponse::ok(m),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn admin_running(
    _auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<crate::domain::match_::Match>> {
    match s.admin_service.list_running().await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn admin_force_abort(
    _auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.admin_service.force_abort(id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}
