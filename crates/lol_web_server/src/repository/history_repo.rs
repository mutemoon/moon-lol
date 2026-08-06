//! 游戏历史 持久层。

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::history::GameHistory;
use crate::domain::{RepoError, RepoResult};

#[async_trait]
pub trait HistoryRepo: Send + Sync {
    async fn list_by_user(&self, user_id: i32) -> RepoResult<Vec<GameHistory>>;
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<GameHistory>>;
    async fn insert(&self, user_id: i32, history: &GameHistory) -> RepoResult<GameHistory>;
    async fn delete(&self, id: Uuid) -> RepoResult<()>;
}

pub struct PgHistoryRepo {
    pub pool: PgPool,
}

const SELECT_COLS: &str = "id, user_id, datetime, game_duration, agents, histories, created_at";

fn parse_row(r: &sqlx::postgres::PgRow) -> RepoResult<GameHistory> {
    Ok(GameHistory {
        id: r.try_get("id")?,
        user_id: r.try_get("user_id")?,
        datetime: r.try_get("datetime")?,
        game_duration: r.try_get("game_duration")?,
        agents: r.try_get("agents")?,
        histories: r.try_get("histories")?,
        created_at: r.try_get("created_at")?,
    })
}

#[async_trait]
impl HistoryRepo for PgHistoryRepo {
    async fn list_by_user(&self, user_id: i32) -> RepoResult<Vec<GameHistory>> {
        let sql = format!(
            "SELECT {SELECT_COLS} FROM game_histories WHERE user_id = $1 ORDER BY created_at DESC"
        );
        let rows = sqlx::query(&sql)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(parse_row).collect()
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<GameHistory>> {
        let sql = format!("SELECT {SELECT_COLS} FROM game_histories WHERE id = $1");
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(ref r) => Ok(Some(parse_row(r)?)),
            None => Ok(None),
        }
    }

    async fn insert(&self, user_id: i32, history: &GameHistory) -> RepoResult<GameHistory> {
        let sql = format!(
            "INSERT INTO game_histories (id, user_id, datetime, game_duration, agents, histories) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING {SELECT_COLS}"
        );
        let row = sqlx::query(&sql)
            .bind(history.id)
            .bind(user_id)
            .bind(history.datetime)
            .bind(history.game_duration)
            .bind(&history.agents)
            .bind(&history.histories)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                if let sqlx::Error::Database(ref db) = e {
                    if db.is_foreign_key_violation() {
                        return RepoError::ForeignKeyViolation;
                    }
                }
                RepoError::Db(e)
            })?;
        parse_row(&row)
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM game_histories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
}
