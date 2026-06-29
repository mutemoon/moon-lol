//! AI Config 路由：获取 / 设置用户 AI 配置。

use axum::Json;
use axum::extract::State;

use super::response::ApiResponse;
use super::{AppState, AuthUser};

pub async fn get_config(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<crate::domain::config::AiConfig> {
    match s.config_service.get_config(auth.user_id).await {
        Ok(cfg) => ApiResponse::ok(cfg),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn set_config(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(cfg): Json<crate::domain::config::AiConfig>,
) -> ApiResponse<()> {
    match s.config_service.set_config(auth.user_id, cfg).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}
