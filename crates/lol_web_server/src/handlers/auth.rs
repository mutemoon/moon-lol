//! Auth 路由：注册 / 登录 / 验证码登录 / 重置密码 / me。

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use super::response::ApiResponse;
use super::{AppState, AuthUser};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub phone: String,
    pub password: String,
    pub code: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: AuthUserDto,
}

#[derive(Serialize, Deserialize)]
pub struct AuthUserDto {
    pub id: i32,
    pub phone: String,
}

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
            user: AuthUserDto {
                id: result.user.id,
                phone: result.user.phone,
            },
        }),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub phone: String,
    pub password: String,
}

pub async fn auth_login(
    State(s): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResponse<AuthResponse> {
    match s.user_service.login(&req.phone, &req.password).await {
        Ok(result) => ApiResponse::ok(AuthResponse {
            token: result.token,
            user: AuthUserDto {
                id: result.user.id,
                phone: result.user.phone,
            },
        }),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Deserialize)]
pub struct CodeLoginRequest {
    pub phone: String,
    pub code: String,
}

pub async fn auth_code_login(
    State(s): State<AppState>,
    Json(req): Json<CodeLoginRequest>,
) -> ApiResponse<AuthResponse> {
    match s.user_service.login_with_code(&req.phone, &req.code).await {
        Ok(result) => ApiResponse::ok(AuthResponse {
            token: result.token,
            user: AuthUserDto {
                id: result.user.id,
                phone: result.user.phone,
            },
        }),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub phone: String,
    pub new_password: String,
    pub code: String,
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
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn auth_me(
    auth: AuthUser,
    headers: axum::http::HeaderMap,
    State(s): State<AppState>,
) -> ApiResponse<AuthUserDto> {
    use axum::http::header::AUTHORIZATION;
    if let Some(token) = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer ").or(Some(v)))
    {
        if let Ok(user) = s.user_service.verify_token(token).await {
            return ApiResponse::ok(AuthUserDto {
                id: user.id,
                phone: user.phone,
            });
        }
    }
    ApiResponse::ok(AuthUserDto {
        id: auth.user_id,
        phone: String::new(),
    })
}
