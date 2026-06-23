//! Config 子系统的持久层（ai_config 表）。
//!
//! 黄金示例：repo trait 是 mock 边界，Pg impl 翻译为 SQL。

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::RepoResult;
use crate::domain::config::AiConfig;

/// Config 持久层 trait。每个方法对应一条数据访问语义。
#[async_trait]
pub trait ConfigRepo: Send + Sync {
    /// 读取用户的配置；不存在返回 None。
    async fn find_by_user(&self, user_id: i32) -> RepoResult<Option<AiConfig>>;

    /// 插入或更新（upsert）用户的配置。
    async fn upsert(&self, user_id: i32, config: &AiConfig) -> RepoResult<()>;
}

/// Postgres 实现。
pub struct PgConfigRepo {
    pub pool: PgPool,
}

#[async_trait]
impl ConfigRepo for PgConfigRepo {
    async fn find_by_user(&self, user_id: i32) -> RepoResult<Option<AiConfig>> {
        let row =
            sqlx::query("SELECT api_key, base_url, preamble FROM ai_config WHERE user_id = $1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;

        match row {
            Some(r) => Ok(Some(AiConfig {
                api_key: r.try_get("api_key")?,
                base_url: r.try_get("base_url")?,
                preamble: r.try_get("preamble")?,
            })),
            None => Ok(None),
        }
    }

    async fn upsert(&self, user_id: i32, config: &AiConfig) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO ai_config (user_id, api_key, base_url, preamble) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (user_id) DO UPDATE \
             SET api_key = EXCLUDED.api_key, base_url = EXCLUDED.base_url, preamble = EXCLUDED.preamble",
        )
        .bind(user_id)
        .bind(&config.api_key)
        .bind(&config.base_url)
        .bind(&config.preamble)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
