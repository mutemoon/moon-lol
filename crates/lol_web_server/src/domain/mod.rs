// 共享错误类型：所有 service / repository 层复用。
// 对应 docs/API_DESIGN.md §3.1 错误模型。

use thiserror::Error;

pub mod agent;
pub mod agent_snapshot;
pub mod auth;
pub mod community;
pub mod config;
pub mod essence;
pub mod local_game;
pub mod match_;
pub mod rank;
pub mod room;
pub mod scenario;
pub mod solo_rules;
pub mod spawn_preset;

/// 服务层错误：业务逻辑错误，会被 handler 映射为 HTTP 状态码。
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("未认证")]
    Unauthorized,

    #[error("无权操作该资源")]
    Forbidden,

    #[error("资源不存在")]
    NotFound,

    #[error("请求参数校验失败: {0}")]
    Validation(String),

    #[error("状态冲突: {0}")]
    Conflict(String),

    #[error("Agent 槽位已达上限 (当前 {current}/上限 {limit})")]
    AgentSlotLimit { current: usize, limit: usize },

    #[error("精粹不足 (需要 {required}, 余额 {balance})")]
    InsufficientEssence { required: i64, balance: i64 },

    #[error("限流")]
    RateLimited,

    #[error("内部错误: {0}")]
    Internal(String),
}

/// 持久层错误：数据库操作错误。
#[derive(Debug, Error)]
pub enum RepoError {
    #[error("数据库错误: {0}")]
    Db(#[from] sqlx::Error),

    #[error("唯一约束冲突")]
    UniqueViolation,

    #[error("外键约束冲突")]
    ForeignKeyViolation,

    #[error("行未找到")]
    NotFound,

    #[error("序列化错误: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("内部错误: {0}")]
    Internal(String),
}

/// 把 ServiceError 映射为 HTTP 状态码（handler 层用）。
impl ServiceError {
    pub fn status_code(&self) -> u16 {
        match self {
            ServiceError::Unauthorized => 401,
            ServiceError::Forbidden => 403,
            ServiceError::NotFound => 404,
            ServiceError::Validation(_) => 400,
            ServiceError::Conflict(_) => 409,
            ServiceError::AgentSlotLimit { .. } | ServiceError::InsufficientEssence { .. } => 402,
            ServiceError::RateLimited => 429,
            ServiceError::Internal(_) => 500,
        }
    }

    /// 错误码字符串（响应体里的 error.code 字段）。
    pub fn code(&self) -> &'static str {
        match self {
            ServiceError::Unauthorized => "UNAUTHORIZED",
            ServiceError::Forbidden => "FORBIDDEN",
            ServiceError::NotFound => "NOT_FOUND",
            ServiceError::Validation(_) => "VALIDATION_FAILED",
            ServiceError::Conflict(_) => "CONFLICT",
            ServiceError::AgentSlotLimit { .. } => "AGENT_SLOT_LIMIT",
            ServiceError::InsufficientEssence { .. } => "INSUFFICIENT_ESSENCE",
            ServiceError::RateLimited => "RATE_LIMITED",
            ServiceError::Internal(_) => "INTERNAL",
        }
    }
}

/// repo 错误转 service 错误的默认映射。
impl From<RepoError> for ServiceError {
    fn from(e: RepoError) -> Self {
        match e {
            RepoError::NotFound => ServiceError::NotFound,
            RepoError::UniqueViolation => ServiceError::Conflict("唯一约束冲突".to_string()),
            RepoError::ForeignKeyViolation => {
                ServiceError::Validation("引用的资源不存在".to_string())
            }
            RepoError::Db(e) => ServiceError::Internal(e.to_string()),
            RepoError::Serde(e) => ServiceError::Internal(e.to_string()),
            RepoError::Internal(msg) => ServiceError::Internal(msg),
        }
    }
}

/// 便捷 Result 别名。
pub type ServiceResult<T> = Result<T, ServiceError>;
pub type RepoResult<T> = Result<T, RepoError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_code_mapping() {
        assert_eq!(ServiceError::Unauthorized.status_code(), 401);
        assert_eq!(ServiceError::Forbidden.status_code(), 403);
        assert_eq!(ServiceError::NotFound.status_code(), 404);
        assert_eq!(ServiceError::Validation("x".into()).status_code(), 400);
        assert_eq!(ServiceError::Conflict("x".into()).status_code(), 409);
        assert_eq!(
            ServiceError::AgentSlotLimit {
                current: 5,
                limit: 5
            }
            .status_code(),
            402
        );
        assert_eq!(
            ServiceError::InsufficientEssence {
                required: 100,
                balance: 50
            }
            .status_code(),
            402
        );
        assert_eq!(ServiceError::RateLimited.status_code(), 429);
        assert_eq!(ServiceError::Internal("x".into()).status_code(), 500);
    }

    #[test]
    fn code_mapping() {
        assert_eq!(ServiceError::Unauthorized.code(), "UNAUTHORIZED");
        assert_eq!(
            ServiceError::AgentSlotLimit {
                current: 0,
                limit: 0
            }
            .code(),
            "AGENT_SLOT_LIMIT"
        );
    }

    #[test]
    fn repo_not_found_maps_to_service_not_found() {
        let svc: ServiceError = RepoError::NotFound.into();
        assert!(matches!(svc, ServiceError::NotFound));
    }

    #[test]
    fn repo_unique_violation_maps_to_conflict() {
        let svc: ServiceError = RepoError::UniqueViolation.into();
        assert!(matches!(svc, ServiceError::Conflict(_)));
    }
}
