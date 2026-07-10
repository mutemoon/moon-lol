//! Agent 子系统的持久层。

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::agent::{Agent, AgentInput};
use crate::domain::spawn_preset::Visibility;
use crate::domain::{RepoError, RepoResult};

#[async_trait]
pub trait AgentRepo: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Agent>>;
    async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<Agent>>;
    async fn list_public(&self) -> RepoResult<Vec<Agent>>;
    async fn find_by_id_with_owner_check(
        &self,
        id: Uuid,
        owner_id: i32,
    ) -> RepoResult<Option<Agent>>;
    async fn insert(&self, owner_id: i32, input: &AgentInput) -> RepoResult<Agent>;
    async fn update(&self, id: Uuid, input: &AgentInput) -> RepoResult<()>;
    async fn update_visibility(&self, id: Uuid, visibility: Visibility) -> RepoResult<()>;
    /// 设置 Fork 溯源关系：forked_from（来源）与 upstream_agent_id（上游同步目标）。
    async fn set_fork_linkage(
        &self,
        id: Uuid,
        forked_from: Option<Uuid>,
        upstream: Option<Uuid>,
    ) -> RepoResult<()>;
    async fn delete(&self, id: Uuid) -> RepoResult<()>;
    async fn count_by_owner(&self, owner_id: i32) -> RepoResult<i64>;
}

pub struct PgAgentRepo {
    pub pool: PgPool,
}

const SELECT_COLS: &str = "id, owner_id, name, champion, agent_type, prompt, model, \
     config_json, visibility, forked_from, upstream_agent_id";

fn parse_row(r: &sqlx::postgres::PgRow) -> RepoResult<Agent> {
    let vis_str: String = r.try_get("visibility")?;
    let visibility = Visibility::from_str(&vis_str)
        .ok_or_else(|| RepoError::Internal(format!("unknown visibility: {vis_str}")))?;
    let type_str: String = r.try_get("agent_type")?;
    let agent_type = crate::domain::agent::AgentType::from_str(&type_str)
        .ok_or_else(|| RepoError::Internal(format!("unknown agent_type: {type_str}")))?;
    Ok(Agent {
        id: r.try_get("id")?,
        owner_id: r.try_get("owner_id")?,
        name: r.try_get("name")?,
        champion: r.try_get("champion")?,
        agent_type,
        prompt: r.try_get("prompt")?,
        model: r.try_get("model")?,
        config_json: r.try_get("config_json")?,
        visibility,
        forked_from: r.try_get("forked_from")?,
        upstream_agent_id: r.try_get("upstream_agent_id")?,
    })
}

#[async_trait]
impl AgentRepo for PgAgentRepo {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Agent>> {
        let sql = format!("SELECT {SELECT_COLS} FROM agents WHERE id = $1");
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(ref r) => Ok(Some(parse_row(r)?)),
            None => Ok(None),
        }
    }

    async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<Agent>> {
        let sql = format!("SELECT {SELECT_COLS} FROM agents WHERE owner_id = $1 ORDER BY name");
        let rows = sqlx::query(&sql)
            .bind(owner_id)
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(parse_row).collect()
    }

    async fn list_public(&self) -> RepoResult<Vec<Agent>> {
        let sql = format!(
            "SELECT {SELECT_COLS} FROM agents WHERE visibility = 'public' ORDER BY name LIMIT 100"
        );
        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;
        rows.iter().map(parse_row).collect()
    }

    async fn find_by_id_with_owner_check(
        &self,
        id: Uuid,
        owner_id: i32,
    ) -> RepoResult<Option<Agent>> {
        let sql = format!("SELECT {SELECT_COLS} FROM agents WHERE id = $1 AND owner_id = $2");
        let row = sqlx::query(&sql)
            .bind(id)
            .bind(owner_id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(ref r) => Ok(Some(parse_row(r)?)),
            None => Ok(None),
        }
    }

    async fn insert(&self, owner_id: i32, input: &AgentInput) -> RepoResult<Agent> {
        let id = Uuid::new_v4();
        let sql = format!(
            "INSERT INTO agents \
             (id, owner_id, name, champion, agent_type, prompt, model, config_json, visibility) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             RETURNING {SELECT_COLS}"
        );
        let row = sqlx::query(&sql)
            .bind(id)
            .bind(owner_id)
            .bind(&input.name)
            .bind(&input.champion)
            .bind(input.agent_type.as_str())
            .bind(&input.prompt)
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
                    if db.is_foreign_key_violation() {
                        return RepoError::ForeignKeyViolation;
                    }
                }
                RepoError::Db(e)
            })?;
        parse_row(&row)
    }

    async fn update(&self, id: Uuid, input: &AgentInput) -> RepoResult<()> {
        let result = sqlx::query(
            "UPDATE agents SET name = $1, champion = $2, agent_type = $3, \
             prompt = $4, model = $5, config_json = $6, \
             visibility = $7, updated_at = CURRENT_TIMESTAMP WHERE id = $8",
        )
        .bind(&input.name)
        .bind(&input.champion)
        .bind(input.agent_type.as_str())
        .bind(&input.prompt)
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

    async fn update_visibility(&self, id: Uuid, visibility: Visibility) -> RepoResult<()> {
        let result = sqlx::query("UPDATE agents SET visibility = $1 WHERE id = $2")
            .bind(visibility.as_str())
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn set_fork_linkage(
        &self,
        id: Uuid,
        forked_from: Option<Uuid>,
        upstream: Option<Uuid>,
    ) -> RepoResult<()> {
        let result =
            sqlx::query("UPDATE agents SET forked_from = $1, upstream_agent_id = $2 WHERE id = $3")
                .bind(forked_from)
                .bind(upstream)
                .bind(id)
                .execute(&self.pool)
                .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM agents WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn count_by_owner(&self, owner_id: i32) -> RepoResult<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agents WHERE owner_id = $1")
            .bind(owner_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }
}
