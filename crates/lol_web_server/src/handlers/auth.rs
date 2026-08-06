//! Auth 路由：注册 / 登录 / 验证码登录 / 重置密码 / me。

use axum::Json;
use axum::extract::State;
use lol_web_protocol::auth::UserInfo;
// 导入并重新导出（保持 handlers::auth::Xxx 路径不变，供外部引用）
pub use lol_web_protocol::auth::{
    AuthResponse, CodeLoginRequest, LoginRequest, RegisterRequest, ResetPasswordRequest,
    UserInfo as AuthUserDto,
};

use super::response::ApiResponse;
use super::{AppState, AuthUser};

pub async fn auth_register(
    State(s): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> ApiResponse<AuthResponse> {
    match s
        .user_service
        .register(&req.phone, &req.password, &req.code)
        .await
    {
        Ok(result) => ApiResponse::ok(AuthResponse {
            token: result.token,
            user: UserInfo {
                id: result.user.id,
                phone: result.user.phone,
            },
        }),
        Err(e) => super::response::api_error(e),
    }
}

pub async fn auth_login(
    State(s): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResponse<AuthResponse> {
    match s.user_service.login(&req.phone, &req.password).await {
        Ok(result) => ApiResponse::ok(AuthResponse {
            token: result.token,
            user: UserInfo {
                id: result.user.id,
                phone: result.user.phone,
            },
        }),
        Err(e) => super::response::api_error(e),
    }
}

pub async fn auth_code_login(
    State(s): State<AppState>,
    Json(req): Json<CodeLoginRequest>,
) -> ApiResponse<AuthResponse> {
    match s.user_service.login_with_code(&req.phone, &req.code).await {
        Ok(result) => ApiResponse::ok(AuthResponse {
            token: result.token,
            user: UserInfo {
                id: result.user.id,
                phone: result.user.phone,
            },
        }),
        Err(e) => super::response::api_error(e),
    }
}

pub async fn auth_reset_password(
    State(s): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> ApiResponse<()> {
    match s
        .user_service
        .reset_password(&req.phone, &req.new_password, &req.code)
        .await
    {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => super::response::api_error(e),
    }
}

pub async fn auth_me(
    auth: AuthUser,
    headers: axum::http::HeaderMap,
    State(s): State<AppState>,
) -> ApiResponse<UserInfo> {
    use axum::http::header::AUTHORIZATION;
    if let Some(token) = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer ").or(Some(v)))
    {
        if let Ok(user) = s.user_service.verify_token(token).await {
            return ApiResponse::ok(UserInfo {
                id: user.id,
                phone: user.phone,
            });
        }
    }
    ApiResponse::ok(UserInfo {
        id: auth.user_id,
        phone: String::new(),
    })
}
