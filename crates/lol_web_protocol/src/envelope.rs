//! 统一响应信封 `ApiResponse<T>` / `ApiError`（纯 serde 类型，无 axum 依赖）。
//!
//! IntoResponse impl 由可选的 `axum` feature 提供，仅 `lol_web_server` 启用。

use serde::{Deserialize, Serialize};

// ── 错误码常量 ──

pub const ERROR_UNAUTHORIZED: &str = "UNAUTHORIZED";
pub const ERROR_FORBIDDEN: &str = "FORBIDDEN";
pub const ERROR_NOT_FOUND: &str = "NOT_FOUND";
pub const ERROR_VALIDATION_FAILED: &str = "VALIDATION_FAILED";
pub const ERROR_CONFLICT: &str = "CONFLICT";
pub const ERROR_AGENT_SLOT_LIMIT: &str = "AGENT_SLOT_LIMIT";
pub const ERROR_INSUFFICIENT_ESSENCE: &str = "INSUFFICIENT_ESSENCE";
pub const ERROR_RATE_LIMITED: &str = "RATE_LIMITED";
pub const ERROR_INTERNAL: &str = "INTERNAL";

// ── 信封类型 ──

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse<T> {
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
        }
    }

    pub fn from_error_parts(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            data: None,
            error: Some(ApiError {
                code: code.into(),
                message: message.into(),
            }),
        }
    }

    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }

    pub fn is_err(&self) -> bool {
        self.error.is_some()
    }
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

// ── 可选的 axum IntoResponse impl（由 lol_web_server 通过 feature = "axum" 启用） ──

#[cfg(feature = "axum")]
mod axum_impl {
    use axum::Json;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    use serde::Serialize;

    use super::ApiResponse;

    impl<T: Serialize> IntoResponse for ApiResponse<T> {
        fn into_response(self) -> Response {
            let status = match &self.error {
                Some(e) => status_from_code(&e.code),
                None => StatusCode::OK,
            };
            (status, Json(self)).into_response()
        }
    }

    fn status_from_code(code: &str) -> StatusCode {
        match code {
            super::ERROR_UNAUTHORIZED => StatusCode::UNAUTHORIZED,
            super::ERROR_FORBIDDEN => StatusCode::FORBIDDEN,
            super::ERROR_NOT_FOUND => StatusCode::NOT_FOUND,
            super::ERROR_VALIDATION_FAILED => StatusCode::BAD_REQUEST,
            super::ERROR_CONFLICT => StatusCode::CONFLICT,
            super::ERROR_AGENT_SLOT_LIMIT | super::ERROR_INSUFFICIENT_ESSENCE => {
                StatusCode::PAYMENT_REQUIRED
            }
            super::ERROR_RATE_LIMITED => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
