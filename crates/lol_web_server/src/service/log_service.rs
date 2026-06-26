//! Log 子系统的 service 层（读取 Bevy 写的 SQLite debug.db）。
//!
//! 这是只读消费端：Bevy 进程（lol_core）写日志到 SQLite，此 service 读取。
//! 远程服务器没有本地 Bevy 的 debug.db，此 service 主要供 desktop 使用。

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

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

/// 以只读模式查询 Bevy 的 SQLite 日志文件。
pub struct SqliteLogReader {
    pub db_path: PathBuf,
}

impl SqliteLogReader {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    async fn open_pool(&self) -> Option<SqlitePool> {
        if !self.db_path.exists() {
            return None;
        }
        let opts = SqliteConnectOptions::new()
            .filename(&self.db_path)
            .read_only(true)
            .immutable(true);
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .ok()
    }
}

#[async_trait]
impl LogReader for SqliteLogReader {
    async fn query_entities(&self) -> ServiceResult<Vec<serde_json::Value>> {
        let pool = match self.open_pool().await {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };

        let rows = sqlx::query(
            "SELECT DISTINCT entity_id, entity_name FROM logs \
             WHERE entity_id IS NOT NULL ORDER BY entity_id",
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            let entity_id: Option<i64> = row
                .try_get(0)
                .map_err(|e| ServiceError::Internal(e.to_string()))?;
            let entity_name: Option<String> = row
                .try_get(1)
                .map_err(|e| ServiceError::Internal(e.to_string()))?;
            result.push(serde_json::json!({
                "entity_id": entity_id,
                "entity_name": entity_name,
            }));
        }
        Ok(result)
    }

    async fn query_categories(&self) -> ServiceResult<Vec<serde_json::Value>> {
        let pool = match self.open_pool().await {
            Some(p) => p,
            None => return Ok(Vec::new()),
        };

        let rows = sqlx::query(
            "SELECT DISTINCT category FROM logs WHERE category IS NOT NULL ORDER BY category",
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            let category: Option<String> = row
                .try_get(0)
                .map_err(|e| ServiceError::Internal(e.to_string()))?;
            result.push(serde_json::json!({
                "category": category,
            }));
        }
        Ok(result)
    }

    async fn query_logs(&self, params: QueryLogsParams) -> ServiceResult<QueryLogsResult> {
        let pool = match self.open_pool().await {
            Some(p) => p,
            None => {
                return Ok(QueryLogsResult {
                    rows: Vec::new(),
                    total_count: 0,
                });
            }
        };

        let limit = params.limit as i64;

        // ── 构造 WHERE 子句与绑定参数（全部走 bind，不内联，防注入）──
        let mut where_clause = String::from("WHERE 1=1");
        let mut levels_args: Vec<String> = vec![];
        let mut entity_id_arg: Option<i64> = None;
        let mut category_arg: Option<String> = None;
        let mut search_arg: Option<String> = None;

        if let Some(ref lvl) = params.levels {
            if !lvl.is_empty() {
                let placeholders: Vec<&str> = lvl.iter().map(|_| "?").collect();
                where_clause.push_str(&format!(" AND level IN ({})", placeholders.join(",")));
                levels_args = lvl.clone();
            }
        }
        if let Some(eid) = params.entity_id {
            where_clause.push_str(" AND entity_id = ?");
            entity_id_arg = Some(eid);
        }
        if let Some(ref cat) = params.category {
            where_clause.push_str(" AND category = ?");
            category_arg = Some(cat.clone());
        }
        if let Some(ref search) = params.search_text {
            let search = search.trim();
            if !search.is_empty() {
                where_clause.push_str(" AND message LIKE ?");
                search_arg = Some(format!("%{search}%"));
            }
        }

        // 1. COUNT
        let sql_count = format!("SELECT COUNT(*) FROM logs {where_clause}");
        let mut q_count = sqlx::query_scalar::<_, i64>(&sql_count);
        for l in &levels_args {
            q_count = q_count.bind(l);
        }
        if let Some(eid) = entity_id_arg {
            q_count = q_count.bind(eid);
        }
        if let Some(ref cat) = category_arg {
            q_count = q_count.bind(cat);
        }
        if let Some(ref search) = search_arg {
            q_count = q_count.bind(search);
        }
        let total_count: i64 = q_count
            .fetch_one(&pool)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // 2. 负 offset 自动算最后一页
        let mut real_offset = params.offset;
        if real_offset < 0 {
            real_offset = std::cmp::max(0, total_count - limit);
        }

        // 3. 数据查询
        let sql_data = format!(
            "SELECT id, timestamp, level, file, line, entity_id, entity_name, category, message \
             FROM logs {where_clause} ORDER BY id ASC LIMIT ? OFFSET ?"
        );
        let mut q_data = sqlx::query(&sql_data);
        for l in &levels_args {
            q_data = q_data.bind(l);
        }
        if let Some(eid) = entity_id_arg {
            q_data = q_data.bind(eid);
        }
        if let Some(ref cat) = category_arg {
            q_data = q_data.bind(cat);
        }
        if let Some(ref search) = search_arg {
            q_data = q_data.bind(search);
        }
        q_data = q_data.bind(limit);
        q_data = q_data.bind(real_offset);

        let rows = q_data
            .fetch_all(&pool)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            let line: Option<i64> = row
                .try_get(4)
                .map_err(|e| ServiceError::Internal(e.to_string()))?;
            let entity_id: Option<i64> = row
                .try_get(5)
                .map_err(|e| ServiceError::Internal(e.to_string()))?;
            result.push(LogRow {
                id: row
                    .try_get(0)
                    .map_err(|e| ServiceError::Internal(e.to_string()))?,
                timestamp: row
                    .try_get(1)
                    .map_err(|e| ServiceError::Internal(e.to_string()))?,
                level: row
                    .try_get(2)
                    .map_err(|e| ServiceError::Internal(e.to_string()))?,
                file: row
                    .try_get(3)
                    .map_err(|e| ServiceError::Internal(e.to_string()))?,
                line,
                entity_id,
                entity_name: row
                    .try_get(6)
                    .map_err(|e| ServiceError::Internal(e.to_string()))?,
                category: row
                    .try_get(7)
                    .map_err(|e| ServiceError::Internal(e.to_string()))?,
                message: row
                    .try_get(8)
                    .map_err(|e| ServiceError::Internal(e.to_string()))?,
            });
        }
        Ok(QueryLogsResult {
            rows: result,
            total_count,
        })
    }

    async fn clear(&self) -> ServiceResult<()> {
        if !self.db_path.exists() {
            return Ok(());
        }
        let opts = SqliteConnectOptions::new().filename(&self.db_path);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;
        sqlx::query("DELETE FROM logs")
            .execute(&pool)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;
        pool.close().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;

    use super::*;

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
