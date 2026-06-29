//! 统一响应包装 `ApiResponse<T>` / `ApiError`，以及
//! `ServiceError ↔ ApiError` 的反向映射（响应状态码用）。

use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

use crate::domain::ServiceError;

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
        }
    }

    pub fn from_error(e: ServiceError) -> Self {
        Self {
            data: None,
            error: Some(ApiError {
                code: e.code().to_string(),
                message: e.to_string(),
            }),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let status = match &self.error {
            Some(e) => StatusCode::from_u16(ServiceError::from_api_error(e).status_code())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            None => StatusCode::OK,
        };
        (status, Json(self)).into_response()
    }
}

impl ServiceError {
    pub(crate) fn from_api_error(e: &ApiError) -> Self {
        match e.code.as_str() {
            "UNAUTHORIZED" => ServiceError::Unauthorized,
            "FORBIDDEN" => ServiceError::Forbidden,
            "NOT_FOUND" => ServiceError::NotFound,
            "VALIDATION_FAILED" => ServiceError::Validation(e.message.clone()),
            "CONFLICT" => ServiceError::Conflict(e.message.clone()),
            "AGENT_SLOT_LIMIT" => ServiceError::AgentSlotLimit {
                current: 0,
                limit: 0,
            },
            "INSUFFICIENT_ESSENCE" => ServiceError::InsufficientEssence {
                required: 0,
                balance: 0,
            },
            "RATE_LIMITED" => ServiceError::RateLimited,
            _ => ServiceError::Internal(e.message.clone()),
        }
    }
}
