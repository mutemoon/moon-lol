//! AgentSnapshot 子系统的持久层。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::agent_snapshot::AgentSnapshot;
use crate::domain::{RepoError, RepoResult};

#[async_trait]
pub trait AgentSnapshotRepo: Send + Sync {
    async fn insert(
        &self,
        agent_id: Uuid,
        version: i32,
        config_freeze: &serde_json::Value,
    ) -> RepoResult<AgentSnapshot>;
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<AgentSnapshot>>;
    async fn list_by_agent(&self, agent_id: Uuid) -> RepoResult<Vec<AgentSnapshot>>;
    async fn max_version(&self, agent_id: Uuid) -> RepoResult<Option<i32>>;
    async fn find_latest(&self, agent_id: Uuid) -> RepoResult<Option<AgentSnapshot>>;
    async fn delete(&self, id: Uuid) -> RepoResult<()>;
}

pub struct PgAgentSnapshotRepo {
    pub pool: PgPool,
}

const SELECT_COLS: &str = "id, agent_id, version, config_freeze, published_at";

fn parse_row(r: &sqlx::postgres::PgRow) -> RepoResult<AgentSnapshot> {
    Ok(AgentSnapshot {
        id: r.try_get("id")?,
        agent_id: r.try_get("agent_id")?,
        version: r.try_get("version")?,
        config_freeze: r.try_get("config_freeze")?,
        published_at: r.try_get::<DateTime<Utc>, _>("published_at")?,
    })
}

#[async_trait]
impl AgentSnapshotRepo for PgAgentSnapshotRepo {
    async fn insert(
        &self,
        agent_id: Uuid,
        version: i32,
        config_freeze: &serde_json::Value,
    ) -> RepoResult<AgentSnapshot> {
        let id = Uuid::new_v4();
        let sql = format!(
            "INSERT INTO agent_snapshots (id, agent_id, version, config_freeze) \
             VALUES ($1, $2, $3, $4) RETURNING {SELECT_COLS}"
        );
        let row = sqlx::query(&sql)
            .bind(id)
            .bind(agent_id)
            .bind(version)
            .bind(config_freeze)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                if let sqlx::Error::Database(ref db) = e {
                    if db.is_unique_violation() {
                        return RepoError::UniqueViolation;
                    }
                    if db.is_foreign_key_violation() {
                        return RepoError::ForeignKeyViolation;
                    }
                }
                RepoError::Db(e)
            })?;
        parse_row(&row)
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<AgentSnapshot>> {
        let sql = format!("SELECT {SELECT_COLS} FROM agent_snapshots WHERE id = $1");
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(ref r) => Ok(Some(parse_row(r)?)),
            None => Ok(None),
        }
    }

    async fn list_by_agent(&self, agent_id: Uuid) -> RepoResult<Vec<AgentSnapshot>> {
        let sql = format!(
            "SELECT {SELECT_COLS} FROM agent_snapshots WHERE agent_id = $1 ORDER BY version"
        );
        let rows = sqlx::query(&sql)
            .bind(agent_id)
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(parse_row).collect()
    }

    async fn max_version(&self, agent_id: Uuid) -> RepoResult<Option<i32>> {
        let row = sqlx::query("SELECT MAX(version) FROM agent_snapshots WHERE agent_id = $1")
            .bind(agent_id)
            .fetch_one(&self.pool)
            .await?;
        let max: Option<i32> = row.try_get(0)?;
        Ok(max)
    }

    async fn find_latest(&self, agent_id: Uuid) -> RepoResult<Option<AgentSnapshot>> {
        let sql = format!(
            "SELECT {SELECT_COLS} FROM agent_snapshots WHERE agent_id = $1 ORDER BY version DESC LIMIT 1"
        );
        let row = sqlx::query(&sql)
            .bind(agent_id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(ref r) => Ok(Some(parse_row(r)?)),
            None => Ok(None),
        }
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM agent_snapshots WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
}
