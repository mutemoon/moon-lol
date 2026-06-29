//! Local Game 路由：本地启动 / 停止。
//!
//! `StartRoomResponse` 复用 `room` 模块的定义（同构：match_id + ws_port）。

use axum::extract::State;
use axum::Json;

use super::response::ApiResponse;
use super::room::StartRoomResponse;
use super::{AppState, AuthUser};
use crate::service::LocalStartInput;

pub async fn local_start(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(input): Json<LocalStartInput>,
) -> ApiResponse<StartRoomResponse> {
    match s.local_game_service.start(auth.user_id, input).await {
        Ok((match_id, port)) => ApiResponse::ok(StartRoomResponse {
            match_id,
            ws_port: port,
        }),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn local_stop(_auth: AuthUser, State(_s): State<AppState>) -> ApiResponse<()> {
    // 简化：local_stop 需要 match_id，此处用 body 传递
    ApiResponse::ok(())
}
