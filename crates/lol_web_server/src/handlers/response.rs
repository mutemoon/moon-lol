//! 响应包装层：从 `lol_web_protocol` 引用 `ApiResponse<T>` / `ApiError`。
//!
//! `IntoResponse` impl 由协议 crate 的 `axum` feature 提供。

// ── 重新导出（保持 handlers::response::ApiResponse 路径不变） ──
pub use lol_web_protocol::envelope::{ApiError, ApiResponse};

use crate::domain::ServiceError;

/// 从 `ServiceError` 构建错误响应（handler 常用）。
pub fn api_error<T>(e: ServiceError) -> ApiResponse<T> {
    ApiResponse::from_error_parts(e.code(), e.to_string())
}
