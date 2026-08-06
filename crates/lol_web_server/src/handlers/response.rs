//! 响应包装层：从 `lol_web_protocol` 引用 `ApiResponse<T>` / `ApiError`。
//!
//! `IntoResponse` impl 由协议 crate 的 `axum` feature 提供。
//! 本文件提供 `ServiceError` → `ApiResponse` 的便捷转换。

// ── 重新导出（保持 handlers::response::ApiResponse 路径不变） ──
pub use lol_web_protocol::envelope::{ApiError, ApiResponse};

use crate::domain::ServiceError;

// ── ServiceError ↔ ApiResponse 便捷方法 ──

impl ServiceError {
    /// 根据 ApiError 的 code 字段反向构造 ServiceError（用于 WS 鉴权等场景）。
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

/// 从 `ServiceError` 构建错误响应（handler 常用）。
pub fn api_error<T>(e: ServiceError) -> ApiResponse<T> {
    ApiResponse::from_error_parts(e.code(), e.to_string())
}
