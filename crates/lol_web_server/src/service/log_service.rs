//! Log 子系统的 service 层（读取 Bevy 写的 SQLite debug.db）。
//!
//! 这是只读消费端：Bevy 进程（lol_core）写日志到 SQLite，此 service 读取。
//! 远程服务器没有本地 Bevy 的 debug.db，此 service 主要供 desktop 使用。

use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::{ServiceError, ServiceResult};

#[derive(Debug, Clone, serde::Serialize)]
pub struct LogRow {
    pub id: i64,
    pub timestamp: i64,
    pub level: String,
    pub file: Option<String>,
    pub line: Option<i64>,
    pub entity_id: Option<i64>,
    pub entity_name: Option<String>,
    pub category: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct QueryLogsResult {
    pub rows: Vec<LogRow>,
    pub total_count: i64,
}

#[derive(Debug, Clone)]
pub struct QueryLogsParams {
    pub offset: i64,
    pub limit: i64,
    pub levels: Option<Vec<String>>,
    pub entity_id: Option<i64>,
    pub category: Option<String>,
    pub search_text: Option<String>,
}

/// 日志读取抽象（可 mock）。
#[async_trait]
pub trait LogReader: Send + Sync {
    async fn query_entities(&self) -> ServiceResult<Vec<serde_json::Value>>;
    async fn query_categories(&self) -> ServiceResult<Vec<serde_json::Value>>;
    async fn query_logs(&self, params: QueryLogsParams) -> ServiceResult<QueryLogsResult>;
    async fn clear(&self) -> ServiceResult<()>;
}

#[async_trait]
pub trait LogService: Send + Sync {
    async fn entities(&self) -> ServiceResult<Vec<serde_json::Value>>;
    async fn categories(&self) -> ServiceResult<Vec<serde_json::Value>>;
    async fn logs(&self, params: QueryLogsParams) -> ServiceResult<QueryLogsResult>;
    async fn clear(&self) -> ServiceResult<()>;
}

pub struct LogServiceImpl {
    pub reader: Arc<dyn LogReader>,
}

impl LogServiceImpl {
    pub fn new(reader: Arc<dyn LogReader>) -> Self {
        Self { reader }
    }
}

#[async_trait]
impl LogService for LogServiceImpl {
    async fn entities(&self) -> ServiceResult<Vec<serde_json::Value>> {
        self.reader.query_entities().await
    }
    async fn categories(&self) -> ServiceResult<Vec<serde_json::Value>> {
        self.reader.query_categories().await
    }
    async fn logs(&self, params: QueryLogsParams) -> ServiceResult<QueryLogsResult> {
        // 参数校验
        let limit = params.limit.clamp(0, 1000);
        if params.offset < -1 {
            return Err(ServiceError::Validation("offset 不能小于 -1".into()));
        }
        let validated = QueryLogsParams { limit, ..params };
        self.reader.query_logs(validated).await
    }
    async fn clear(&self) -> ServiceResult<()> {
        self.reader.clear().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        pub Reader {}
        #[async_trait]
        impl LogReader for Reader {
            async fn query_entities(&self) -> ServiceResult<Vec<serde_json::Value>>;
            async fn query_categories(&self) -> ServiceResult<Vec<serde_json::Value>>;
            async fn query_logs(&self, params: QueryLogsParams) -> ServiceResult<QueryLogsResult>;
            async fn clear(&self) -> ServiceResult<()>;
        }
    }

    #[tokio::test]
    async fn logs_delegates_to_reader() {
        let mut reader = MockReader::new();
        reader.expect_query_logs().returning(|_| {
            Ok(QueryLogsResult {
                rows: vec![],
                total_count: 0,
            })
        });
        let svc = LogServiceImpl::new(Arc::new(reader));
        let result = svc
            .logs(QueryLogsParams {
                offset: 0,
                limit: 10,
                levels: None,
                entity_id: None,
                category: None,
                search_text: None,
            })
            .await
            .unwrap();
        assert_eq!(result.total_count, 0);
    }

    #[tokio::test]
    async fn logs_rejects_invalid_offset() {
        let reader = MockReader::new();
        let svc = LogServiceImpl::new(Arc::new(reader));
        let err = svc
            .logs(QueryLogsParams {
                offset: -5,
                limit: 10,
                levels: None,
                entity_id: None,
                category: None,
                search_text: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[tokio::test]
    async fn logs_clamps_limit() {
        let mut reader = MockReader::new();
        reader.expect_query_logs().returning(|p| {
            // 验证 limit 被 clamp 到 1000
            assert!(p.limit <= 1000);
            Ok(QueryLogsResult {
                rows: vec![],
                total_count: 0,
            })
        });
        let svc = LogServiceImpl::new(Arc::new(reader));
        svc.logs(QueryLogsParams {
            offset: 0,
            limit: 99999,
            levels: None,
            entity_id: None,
            category: None,
            search_text: None,
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn entities_delegates() {
        let mut reader = MockReader::new();
        reader
            .expect_query_entities()
            .returning(|| Ok(vec![serde_json::json!({"entity_id": 1})]));
        let svc = LogServiceImpl::new(Arc::new(reader));
        assert_eq!(svc.entities().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn clear_delegates() {
        let mut reader = MockReader::new();
        reader.expect_clear().returning(|| Ok(()));
        let svc = LogServiceImpl::new(Arc::new(reader));
        svc.clear().await.unwrap();
    }
}
