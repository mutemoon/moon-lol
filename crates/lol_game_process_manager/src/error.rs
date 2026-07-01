//! 进程托管层错误类型。
//!
//! 不复用 `lol_web_server::domain::ServiceError`（后者与 sqlx / HTTP 状态码耦合）。
//! 云端 `LocalGameService` 在调用本层后，按需把 [`ManagerError`] 映射回 `ServiceError`。

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("请求参数校验失败: {0}")]
    Validation(String),

    #[error("状态冲突: {0}")]
    Conflict(String),

    #[error("资源不存在")]
    NotFound,

    #[error("内部错误: {0}")]
    Internal(String),
}

pub type ManagerResult<T> = Result<T, ManagerError>;
