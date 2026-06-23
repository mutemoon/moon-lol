//! Scenario 子系统的持久层。

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::scenario::{Scenario, ScenarioInput};
use crate::domain::{RepoError, RepoResult};

#[async_trait]
pub trait ScenarioRepo: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Scenario>>;
    async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<Scenario>>;
    async fn insert(&self, owner_id: i32, input: &ScenarioInput) -> RepoResult<Scenario>;
    async fn update(&self, id: Uuid, input: &ScenarioInput) -> RepoResult<()>;
    async fn delete(&self, id: Uuid) -> RepoResult<()>;
    async fn get_win_condition(
        &self,
        owner_id: i32,
        scenario_id: Uuid,
    ) -> RepoResult<Option<serde_json::Value>>;
    async fn save_win_condition(
        &self,
        owner_id: i32,
        scenario_id: Uuid,
        condition: &serde_json::Value,
    ) -> RepoResult<()>;
}

pub struct PgScenarioRepo {
    pub pool: PgPool,
}

const SELECT_COLS: &str = "id, owner_id, name, agents, created_at";

fn parse_row(r: &sqlx::postgres::PgRow) -> RepoResult<Scenario> {
    Ok(Scenario {
        id: r.try_get("id")?,
        owner_id: r.try_get("owner_id")?,
        name: r.try_get("name")?,
        agents: r.try_get("agents")?,
        created_at: r.try_get("created_at")?,
    })
}

#[async_trait]
impl ScenarioRepo for PgScenarioRepo {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Scenario>> {
        let sql = format!("SELECT {SELECT_COLS} FROM scenarios WHERE id = $1");
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(ref r) => Ok(Some(parse_row(r)?)),
            None => Ok(None),
        }
    }

    async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<Scenario>> {
        let sql = format!(
            "SELECT {SELECT_COLS} FROM scenarios WHERE owner_id = $1 ORDER BY created_at DESC"
        );
        let rows = sqlx::query(&sql)
            .bind(owner_id)
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(parse_row).collect()
    }

    async fn insert(&self, owner_id: i32, input: &ScenarioInput) -> RepoResult<Scenario> {
        let id = Uuid::new_v4();
        let sql = format!(
            "INSERT INTO scenarios (id, owner_id, name, agents) VALUES ($1, $2, $3, $4) RETURNING {SELECT_COLS}"
        );
        let row = sqlx::query(&sql)
            .bind(id)
            .bind(owner_id)
            .bind(&input.name)
            .bind(&input.agents)
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

    async fn update(&self, id: Uuid, input: &ScenarioInput) -> RepoResult<()> {
        let result = sqlx::query("UPDATE scenarios SET name = $1, agents = $2 WHERE id = $3")
            .bind(&input.name)
            .bind(&input.agents)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                if let sqlx::Error::Database(ref db) = e {
                    if db.is_unique_violation() {
                        return RepoError::UniqueViolation;
                    }
                }
                RepoError::Db(e)
            })?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM scenarios WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn get_win_condition(
        &self,
        owner_id: i32,
        scenario_id: Uuid,
    ) -> RepoResult<Option<serde_json::Value>> {
        let row = sqlx::query(
            "SELECT condition FROM scenario_win_conditions WHERE owner_id = $1 AND scenario_id = $2",
        )
        .bind(owner_id)
        .bind(scenario_id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(r) => Ok(Some(r.try_get("condition")?)),
            None => Ok(None),
        }
    }

    async fn save_win_condition(
        &self,
        owner_id: i32,
        scenario_id: Uuid,
        condition: &serde_json::Value,
    ) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO scenario_win_conditions (owner_id, scenario_id, condition) \
             VALUES ($1, $2, $3) ON CONFLICT (owner_id, scenario_id) DO UPDATE SET condition = EXCLUDED.condition",
        )
        .bind(owner_id)
        .bind(scenario_id)
        .bind(condition)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
