//! AgentConfig 子系统的持久层。

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::agent_config::{AgentConfig, AgentConfigInput, AgentType};
use crate::domain::spawn_preset::Visibility;
use crate::domain::{RepoError, RepoResult};

#[async_trait]
pub trait AgentConfigRepo: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<AgentConfig>>;
    async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<AgentConfig>>;
    async fn insert(&self, owner_id: i32, input: &AgentConfigInput) -> RepoResult<AgentConfig>;
    async fn update(&self, id: Uuid, input: &AgentConfigInput) -> RepoResult<()>;
    async fn delete(&self, id: Uuid) -> RepoResult<()>;
    async fn count_by_owner(&self, owner_id: i32) -> RepoResult<i64>;
}

pub struct PgAgentConfigRepo {
    pub pool: PgPool,
}

const SELECT_COLS: &str =
    "id, owner_id, name, agent_type, prompt, preamble, model, config_json, visibility, forked_from";

fn parse_row(r: &sqlx::postgres::PgRow) -> RepoResult<AgentConfig> {
    let type_str: String = r.try_get("agent_type")?;
    let vis_str: String = r.try_get("visibility")?;
    let agent_type = AgentType::from_str(&type_str)
        .ok_or_else(|| RepoError::Internal(format!("unknown agent_type: {type_str}")))?;
    let visibility = Visibility::from_str(&vis_str)
        .ok_or_else(|| RepoError::Internal(format!("unknown visibility: {vis_str}")))?;
    Ok(AgentConfig {
        id: r.try_get("id")?,
        owner_id: r.try_get("owner_id")?,
        name: r.try_get("name")?,
        agent_type,
        prompt: r.try_get("prompt")?,
        preamble: r.try_get("preamble")?,
        model: r.try_get("model")?,
        config_json: r.try_get("config_json")?,
        visibility,
        forked_from: r.try_get("forked_from")?,
    })
}

#[async_trait]
impl AgentConfigRepo for PgAgentConfigRepo {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<AgentConfig>> {
        let sql = format!("SELECT {SELECT_COLS} FROM agent_configs WHERE id = $1");
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(ref r) => Ok(Some(parse_row(r)?)),
            None => Ok(None),
        }
    }

    async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<AgentConfig>> {
        let sql =
            format!("SELECT {SELECT_COLS} FROM agent_configs WHERE owner_id = $1 ORDER BY name");
        let rows = sqlx::query(&sql)
            .bind(owner_id)
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(parse_row).collect()
    }

    async fn insert(&self, owner_id: i32, input: &AgentConfigInput) -> RepoResult<AgentConfig> {
        let id = Uuid::new_v4();
        let sql = format!(
            "INSERT INTO agent_configs \
             (id, owner_id, name, agent_type, prompt, preamble, model, config_json, visibility) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             RETURNING {SELECT_COLS}"
        );
        let row = sqlx::query(&sql)
            .bind(id)
            .bind(owner_id)
            .bind(&input.name)
            .bind(input.agent_type.as_str())
            .bind(&input.prompt)
            .bind(&input.preamble)
            .bind(&input.model)
            .bind(&input.config_json)
            .bind(input.visibility.as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                if let sqlx::Error::Database(ref db) = e {
                    if db.is_unique_violation() {
                        return RepoError::UniqueViolation;
                    }
                }
                RepoError::Db(e)
            })?;
        parse_row(&row)
    }

    async fn update(&self, id: Uuid, input: &AgentConfigInput) -> RepoResult<()> {
        let result = sqlx::query(
            "UPDATE agent_configs SET name = $1, agent_type = $2, prompt = $3, \
             preamble = $4, model = $5, config_json = $6, visibility = $7, \
             updated_at = CURRENT_TIMESTAMP WHERE id = $8",
        )
        .bind(&input.name)
        .bind(input.agent_type.as_str())
        .bind(&input.prompt)
        .bind(&input.preamble)
        .bind(&input.model)
        .bind(&input.config_json)
        .bind(input.visibility.as_str())
        .bind(id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM agent_configs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await;
        match result {
            Ok(r) if r.rows_affected() == 0 => Err(RepoError::NotFound),
            Ok(_) => Ok(()),
            Err(e) => {
                if let sqlx::Error::Database(ref db) = e {
                    if db.is_foreign_key_violation() {
                        return Err(RepoError::ForeignKeyViolation);
                    }
                }
                Err(RepoError::Db(e))
            }
        }
    }

    async fn count_by_owner(&self, owner_id: i32) -> RepoResult<i64> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM agent_configs WHERE owner_id = $1")
                .bind(owner_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(count)
    }
}
