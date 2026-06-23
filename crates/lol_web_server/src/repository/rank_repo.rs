//! Rank 子系统的持久层（seasons + rank_queues + elo_ratings）。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::rank::{QueueStatus, SeasonStatus};
use crate::domain::{RepoError, RepoResult};

// ── Season ──

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct Season {
    pub id: Uuid,
    pub name: String,
    pub mode: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub status: SeasonStatus,
}

#[async_trait]
pub trait SeasonRepo: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Season>>;
    async fn find_current(&self, mode: &str) -> RepoResult<Option<Season>>;
    async fn list_by_mode(&self, mode: &str) -> RepoResult<Vec<Season>>;
    async fn insert(&self, s: &NewSeason) -> RepoResult<Season>;
    async fn update_status(&self, id: Uuid, status: SeasonStatus) -> RepoResult<()>;
}

pub struct NewSeason {
    pub name: String,
    pub mode: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

pub struct PgSeasonRepo {
    pub pool: PgPool,
}

fn parse_season(r: &sqlx::postgres::PgRow) -> RepoResult<Season> {
    let status_str: String = r.try_get("status")?;
    let status = SeasonStatus::from_str(&status_str)
        .ok_or_else(|| RepoError::Internal(format!("unknown season status: {status_str}")))?;
    Ok(Season {
        id: r.try_get("id")?,
        name: r.try_get("name")?,
        mode: r.try_get("mode")?,
        starts_at: r.try_get("starts_at")?,
        ends_at: r.try_get("ends_at")?,
        status,
    })
}

#[async_trait]
impl SeasonRepo for PgSeasonRepo {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Season>> {
        let row = sqlx::query(
            "SELECT id, name, mode, starts_at, ends_at, status FROM seasons WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(ref r) => Ok(Some(parse_season(r)?)),
            None => Ok(None),
        }
    }

    async fn find_current(&self, mode: &str) -> RepoResult<Option<Season>> {
        let row = sqlx::query(
            "SELECT id, name, mode, starts_at, ends_at, status FROM seasons \
             WHERE mode = $1 AND status = 'active' ORDER BY starts_at DESC LIMIT 1",
        )
        .bind(mode)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(ref r) => Ok(Some(parse_season(r)?)),
            None => Ok(None),
        }
    }

    async fn list_by_mode(&self, mode: &str) -> RepoResult<Vec<Season>> {
        let rows = sqlx::query(
            "SELECT id, name, mode, starts_at, ends_at, status FROM seasons \
             WHERE mode = $1 ORDER BY starts_at DESC",
        )
        .bind(mode)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(parse_season).collect()
    }

    async fn insert(&self, s: &NewSeason) -> RepoResult<Season> {
        let id = Uuid::new_v4();
        let row = sqlx::query(
            "INSERT INTO seasons (id, name, mode, starts_at, ends_at, status) \
             VALUES ($1, $2, $3, $4, $5, 'scheduled') \
             RETURNING id, name, mode, starts_at, ends_at, status",
        )
        .bind(id)
        .bind(&s.name)
        .bind(&s.mode)
        .bind(s.starts_at)
        .bind(s.ends_at)
        .fetch_one(&self.pool)
        .await?;
        parse_season(&row)
    }

    async fn update_status(&self, id: Uuid, status: SeasonStatus) -> RepoResult<()> {
        let result = sqlx::query("UPDATE seasons SET status = $1 WHERE id = $2")
            .bind(status.as_str())
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
}

// ── RankQueue ──

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct RankQueueEntry {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub agent_snapshot_id: Uuid,
    pub user_id: i32,
    pub mode: String,
    pub season_id: Uuid,
    pub status: QueueStatus,
    pub enqueued_at: DateTime<Utc>,
    pub last_match_at: Option<DateTime<Utc>>,
}

pub struct NewQueueEntry {
    pub agent_id: Uuid,
    pub agent_snapshot_id: Uuid,
    pub user_id: i32,
    pub mode: String,
    pub season_id: Uuid,
}

#[async_trait]
pub trait RankQueueRepo: Send + Sync {
    async fn enqueue(&self, entry: &NewQueueEntry) -> RepoResult<RankQueueEntry>;
    async fn dequeue(&self, agent_id: Uuid) -> RepoResult<()>;
    async fn find_by_agent(&self, agent_id: Uuid) -> RepoResult<Option<RankQueueEntry>>;
    async fn list_by_user(&self, user_id: i32) -> RepoResult<Vec<RankQueueEntry>>;
    async fn list_queued(&self, mode: &str) -> RepoResult<Vec<RankQueueEntry>>;
    async fn update_status(&self, id: Uuid, status: QueueStatus) -> RepoResult<()>;
    /// 找一个 ELO 接近的待匹配对手（排除自己）。
    async fn find_opponent(
        &self,
        mode: &str,
        agent_id: Uuid,
        rating: f64,
        window: f64,
    ) -> RepoResult<Option<RankQueueEntry>>;
}

pub struct PgRankQueueRepo {
    pub pool: PgPool,
}

fn parse_queue(r: &sqlx::postgres::PgRow) -> RepoResult<RankQueueEntry> {
    let status_str: String = r.try_get("status")?;
    let status = QueueStatus::from_str(&status_str)
        .ok_or_else(|| RepoError::Internal(format!("unknown queue status: {status_str}")))?;
    Ok(RankQueueEntry {
        id: r.try_get("id")?,
        agent_id: r.try_get("agent_id")?,
        agent_snapshot_id: r.try_get("agent_snapshot_id")?,
        user_id: r.try_get("user_id")?,
        mode: r.try_get("mode")?,
        season_id: r.try_get("season_id")?,
        status,
        enqueued_at: r.try_get("enqueued_at")?,
        last_match_at: r.try_get("last_match_at")?,
    })
}

const QUEUE_COLS: &str = "id, agent_id, agent_snapshot_id, user_id, mode, season_id, \
     status, enqueued_at, last_match_at";

#[async_trait]
impl RankQueueRepo for PgRankQueueRepo {
    async fn enqueue(&self, entry: &NewQueueEntry) -> RepoResult<RankQueueEntry> {
        let id = Uuid::new_v4();
        let row = sqlx::query(&format!(
            "INSERT INTO rank_queues (id, agent_id, agent_snapshot_id, user_id, mode, season_id, status) \
             VALUES ($1, $2, $3, $4, $5, $6, 'queued') RETURNING {QUEUE_COLS}"
        ))
        .bind(id)
        .bind(entry.agent_id)
        .bind(entry.agent_snapshot_id)
        .bind(entry.user_id)
        .bind(&entry.mode)
        .bind(entry.season_id)
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
        parse_queue(&row)
    }

    async fn dequeue(&self, agent_id: Uuid) -> RepoResult<()> {
        sqlx::query("DELETE FROM rank_queues WHERE agent_id = $1")
            .bind(agent_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_by_agent(&self, agent_id: Uuid) -> RepoResult<Option<RankQueueEntry>> {
        let row = sqlx::query(&format!(
            "SELECT {QUEUE_COLS} FROM rank_queues WHERE agent_id = $1 LIMIT 1"
        ))
        .bind(agent_id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(ref r) => Ok(Some(parse_queue(r)?)),
            None => Ok(None),
        }
    }

    async fn list_by_user(&self, user_id: i32) -> RepoResult<Vec<RankQueueEntry>> {
        let rows = sqlx::query(&format!(
            "SELECT {QUEUE_COLS} FROM rank_queues WHERE user_id = $1 ORDER BY enqueued_at DESC"
        ))
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(parse_queue).collect()
    }

    async fn list_queued(&self, mode: &str) -> RepoResult<Vec<RankQueueEntry>> {
        let rows = sqlx::query(&format!(
            "SELECT {QUEUE_COLS} FROM rank_queues WHERE mode = $1 AND status = 'queued' \
             ORDER BY enqueued_at"
        ))
        .bind(mode)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(parse_queue).collect()
    }

    async fn update_status(&self, id: Uuid, status: QueueStatus) -> RepoResult<()> {
        let result = sqlx::query("UPDATE rank_queues SET status = $1 WHERE id = $2")
            .bind(status.as_str())
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn find_opponent(
        &self,
        mode: &str,
        agent_id: Uuid,
        _rating: f64,
        _window: f64,
    ) -> RepoResult<Option<RankQueueEntry>> {
        // 简化版：按入队时间最早的对手匹配（ELO 过滤待后续优化）
        let row = sqlx::query(&format!(
            "SELECT {QUEUE_COLS} FROM rank_queues \
             WHERE mode = $1 AND status = 'queued' AND agent_id != $2 \
             ORDER BY enqueued_at LIMIT 1"
        ))
        .bind(mode)
        .bind(agent_id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(ref r) => Ok(Some(parse_queue(r)?)),
            None => Ok(None),
        }
    }
}

// ── ELO ──

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct EloRating {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub mode: String,
    pub season_id: Uuid,
    pub rating: f64,
    pub wins: i32,
    pub losses: i32,
    pub draws: i32,
}

#[async_trait]
pub trait EloRepo: Send + Sync {
    async fn find(
        &self,
        agent_id: Uuid,
        mode: &str,
        season_id: Uuid,
    ) -> RepoResult<Option<EloRating>>;
    async fn upsert_initial(
        &self,
        agent_id: Uuid,
        mode: &str,
        season_id: Uuid,
    ) -> RepoResult<EloRating>;
    /// 原子更新 rating + wins/losses/draws。
    async fn update_after_match(
        &self,
        id: Uuid,
        new_rating: f64,
        is_win: bool,
        is_draw: bool,
    ) -> RepoResult<()>;
    async fn leaderboard(
        &self,
        mode: &str,
        season_id: Uuid,
        limit: i64,
    ) -> RepoResult<Vec<EloRating>>;
}

pub struct PgEloRepo {
    pub pool: PgPool,
}

const ELO_COLS: &str = "id, agent_id, mode, season_id, rating, wins, losses, draws";

fn parse_elo(r: &sqlx::postgres::PgRow) -> RepoResult<EloRating> {
    Ok(EloRating {
        id: r.try_get("id")?,
        agent_id: r.try_get("agent_id")?,
        mode: r.try_get("mode")?,
        season_id: r.try_get("season_id")?,
        rating: r.try_get("rating")?,
        wins: r.try_get("wins")?,
        losses: r.try_get("losses")?,
        draws: r.try_get("draws")?,
    })
}

#[async_trait]
impl EloRepo for PgEloRepo {
    async fn find(
        &self,
        agent_id: Uuid,
        mode: &str,
        season_id: Uuid,
    ) -> RepoResult<Option<EloRating>> {
        let row = sqlx::query(&format!(
            "SELECT {ELO_COLS} FROM elo_ratings WHERE agent_id = $1 AND mode = $2 AND season_id = $3"
        ))
        .bind(agent_id)
        .bind(mode)
        .bind(season_id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(ref r) => Ok(Some(parse_elo(r)?)),
            None => Ok(None),
        }
    }

    async fn upsert_initial(
        &self,
        agent_id: Uuid,
        mode: &str,
        season_id: Uuid,
    ) -> RepoResult<EloRating> {
        let id = Uuid::new_v4();
        let row = sqlx::query(&format!(
            "INSERT INTO elo_ratings (id, agent_id, mode, season_id, rating, wins, losses, draws) \
             VALUES ($1, $2, $3, $4, 1200, 0, 0, 0) \
             ON CONFLICT (agent_id, mode, season_id) DO UPDATE SET agent_id = elo_ratings.agent_id \
             RETURNING {ELO_COLS}"
        ))
        .bind(id)
        .bind(agent_id)
        .bind(mode)
        .bind(season_id)
        .fetch_one(&self.pool)
        .await?;
        parse_elo(&row)
    }

    async fn update_after_match(
        &self,
        id: Uuid,
        new_rating: f64,
        is_win: bool,
        is_draw: bool,
    ) -> RepoResult<()> {
        let win_inc = if is_win { 1 } else { 0 };
        let loss_inc = if !is_win && !is_draw { 1 } else { 0 };
        let draw_inc = if is_draw { 1 } else { 0 };
        sqlx::query(
            "UPDATE elo_ratings SET rating = $1, wins = wins + $2, losses = losses + $3, \
             draws = draws + $4, updated_at = CURRENT_TIMESTAMP WHERE id = $5",
        )
        .bind(new_rating)
        .bind(win_inc)
        .bind(loss_inc)
        .bind(draw_inc)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn leaderboard(
        &self,
        mode: &str,
        season_id: Uuid,
        limit: i64,
    ) -> RepoResult<Vec<EloRating>> {
        let rows = sqlx::query(&format!(
            "SELECT {ELO_COLS} FROM elo_ratings WHERE mode = $1 AND season_id = $2 \
             ORDER BY rating DESC LIMIT $3"
        ))
        .bind(mode)
        .bind(season_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(parse_elo).collect()
    }
}
