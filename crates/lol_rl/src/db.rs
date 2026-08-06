use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TaskRow {
    pub id: Uuid,
    pub name: String,
    pub agent_type: String,
    pub env_name: String,
    pub status: String,
    pub config_json: serde_json::Value,
    pub current_step: i64,
    pub ep_return: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CheckpointRow {
    pub id: Uuid,
    pub task_id: Uuid,
    pub step: i64,
    pub path: String,
    pub ep_return: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("数据库错误: {0}")]
    Db(#[from] sqlx::Error),
    #[error("行未找到")]
    NotFound,
    #[error("唯一约束冲突")]
    UniqueViolation,
    #[error("外键约束冲突")]
    ForeignKeyViolation,
}

pub type RepoResult<T> = Result<T, RepoError>;

fn map_db_error(e: sqlx::Error) -> RepoError {
    if let sqlx::Error::Database(ref db) = e {
        if db.is_unique_violation() {
            return RepoError::UniqueViolation;
        }
        if db.is_foreign_key_violation() {
            return RepoError::ForeignKeyViolation;
        }
    }
    RepoError::Db(e)
}

fn parse_task_row(r: &PgRow) -> RepoResult<TaskRow> {
    Ok(TaskRow {
        id: r.try_get("id")?,
        name: r.try_get("name")?,
        agent_type: r.try_get("agent_type")?,
        env_name: r.try_get("env_name")?,
        status: r.try_get("status")?,
        config_json: r.try_get("config_json")?,
        current_step: r.try_get("current_step")?,
        ep_return: r.try_get("ep_return")?,
        created_at: r.try_get("created_at")?,
        updated_at: r.try_get("updated_at")?,
    })
}

fn parse_checkpoint_row(r: &PgRow) -> RepoResult<CheckpointRow> {
    Ok(CheckpointRow {
        id: r.try_get("id")?,
        task_id: r.try_get("task_id")?,
        step: r.try_get("step")?,
        path: r.try_get("path")?,
        ep_return: r.try_get("ep_return")?,
        created_at: r.try_get("created_at")?,
    })
}

#[async_trait]
pub trait RlRepo: Send + Sync {
    async fn insert_task(&self, task: &TaskRow) -> RepoResult<()>;
    async fn list_tasks(&self) -> RepoResult<Vec<TaskRow>>;
    async fn get_task(&self, id: &str) -> RepoResult<Option<TaskRow>>;
    async fn update_status(&self, id: &str, status: &str) -> RepoResult<()>;
    async fn update_progress(&self, id: &str, step: i64, ep_return: f32) -> RepoResult<()>;
    async fn mark_all_running_interrupted(&self) -> RepoResult<usize>;
    async fn insert_checkpoint(&self, cp: &CheckpointRow) -> RepoResult<()>;
    async fn list_checkpoints(&self, task_id: &str) -> RepoResult<Vec<CheckpointRow>>;
    async fn get_checkpoint(&self, task_id: &str, id: &str) -> RepoResult<Option<CheckpointRow>>;
}

pub struct PgRlRepo {
    pub pool: PgPool,
}

#[async_trait]
impl RlRepo for PgRlRepo {
    async fn insert_task(&self, task: &TaskRow) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO rl_tasks (id, name, agent_type, env_name, status, config_json, current_step, ep_return) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(task.id)
        .bind(&task.name)
        .bind(&task.agent_type)
        .bind(&task.env_name)
        .bind(&task.status)
        .bind(&task.config_json)
        .bind(task.current_step)
        .bind(task.ep_return)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(())
    }

    async fn list_tasks(&self) -> RepoResult<Vec<TaskRow>> {
        let rows = sqlx::query("SELECT * FROM rl_tasks ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(parse_task_row).collect()
    }

    async fn get_task(&self, id: &str) -> RepoResult<Option<TaskRow>> {
        let uuid = Uuid::parse_str(id).map_err(|_| RepoError::NotFound)?;
        let row = sqlx::query("SELECT * FROM rl_tasks WHERE id = $1")
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(ref r) => Ok(Some(parse_task_row(r)?)),
            None => Ok(None),
        }
    }

    async fn update_status(&self, id: &str, status: &str) -> RepoResult<()> {
        let uuid = Uuid::parse_str(id).map_err(|_| RepoError::NotFound)?;
        let result = sqlx::query(
            "UPDATE rl_tasks SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
        )
        .bind(status)
        .bind(uuid)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn update_progress(&self, id: &str, step: i64, ep_return: f32) -> RepoResult<()> {
        let uuid = Uuid::parse_str(id).map_err(|_| RepoError::NotFound)?;
        let result = sqlx::query(
            "UPDATE rl_tasks SET current_step = $1, ep_return = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $3",
        )
        .bind(step)
        .bind(ep_return)
        .bind(uuid)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn mark_all_running_interrupted(&self) -> RepoResult<usize> {
        let result = sqlx::query(
            "UPDATE rl_tasks SET status = 'interrupted', updated_at = CURRENT_TIMESTAMP WHERE status = 'running'",
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() as usize)
    }

    async fn insert_checkpoint(&self, cp: &CheckpointRow) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO rl_checkpoints (id, task_id, step, path, ep_return) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(cp.id)
        .bind(cp.task_id)
        .bind(cp.step)
        .bind(&cp.path)
        .bind(cp.ep_return)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(())
    }

    async fn list_checkpoints(&self, task_id: &str) -> RepoResult<Vec<CheckpointRow>> {
        let uuid = Uuid::parse_str(task_id).map_err(|_| RepoError::NotFound)?;
        let rows =
            sqlx::query("SELECT * FROM rl_checkpoints WHERE task_id = $1 ORDER BY step DESC")
                .bind(uuid)
                .fetch_all(&self.pool)
                .await?;
        rows.iter().map(parse_checkpoint_row).collect()
    }

    async fn get_checkpoint(&self, task_id: &str, id: &str) -> RepoResult<Option<CheckpointRow>> {
        let task_uuid = Uuid::parse_str(task_id).map_err(|_| RepoError::NotFound)?;
        let cp_uuid = Uuid::parse_str(id).map_err(|_| RepoError::NotFound)?;
        let row = sqlx::query("SELECT * FROM rl_checkpoints WHERE task_id = $1 AND id = $2")
            .bind(task_uuid)
            .bind(cp_uuid)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(ref r) => Ok(Some(parse_checkpoint_row(r)?)),
            None => Ok(None),
        }
    }
}

pub async fn apply_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    let sql = include_str!("../migrations/schema.sql");
    for stmt in sql
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with("--"))
    {
        sqlx::query(stmt).execute(pool).await?;
    }
    Ok(())
}
