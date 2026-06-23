//! Match 子系统的持久层（matches + match_participants + match_events）。

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::match_::{
    Match, MatchEvent, MatchForm, MatchParticipant, MatchStatus, ParticipantResult, Winner,
};
use crate::domain::spawn_preset::Team;
use crate::domain::{RepoError, RepoResult};

#[derive(Debug, Clone)]
pub struct MatchInput {
    pub form: MatchForm,
    pub room_id: Option<Uuid>,
    pub mode: String,
    pub scenario_id: Option<Uuid>,
    pub win_condition: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ParticipantInput {
    pub agent_snapshot_id: Uuid,
    pub agent_id: Uuid,
    pub user_id: i32,
    pub team: Team,
}

#[derive(Debug, Clone)]
pub struct MatchEventInput {
    pub event_type: String,
    pub agent_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub game_time_ms: i64,
}

const MATCH_COLS: &str =
    "id, form, room_id, owner_id, mode, status, bevy_port, winner_team, abort_reason";

fn parse_match(r: &sqlx::postgres::PgRow) -> RepoResult<Match> {
    let form_str: String = r.try_get("form")?;
    let form = MatchForm::from_str(&form_str)
        .ok_or_else(|| RepoError::Internal(format!("unknown match form: {form_str}")))?;
    let status_str: String = r.try_get("status")?;
    let status = MatchStatus::from_str(&status_str)
        .ok_or_else(|| RepoError::Internal(format!("unknown match status: {status_str}")))?;
    let winner_str: Option<String> = r.try_get("winner_team")?;
    let winner_team = winner_str
        .map(|s| {
            Winner::from_str(&s)
                .ok_or_else(|| RepoError::Internal(format!("unknown winner_team: {s}")))
        })
        .transpose()?;
    Ok(Match {
        id: r.try_get("id")?,
        form,
        room_id: r.try_get("room_id")?,
        owner_id: r.try_get("owner_id")?,
        mode: r.try_get("mode")?,
        status,
        bevy_port: r.try_get("bevy_port")?,
        winner_team,
        abort_reason: r.try_get("abort_reason")?,
    })
}

#[async_trait]
pub trait MatchRepo: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Match>>;
    async fn list_by_owner(&self, owner_id: i32, limit: i64) -> RepoResult<Vec<Match>>;
    async fn list_by_status(&self, status: MatchStatus, limit: i64) -> RepoResult<Vec<Match>>;
    async fn insert(&self, owner_id: i32, input: &MatchInput) -> RepoResult<Match>;
    async fn update_status(&self, id: Uuid, from: MatchStatus, to: MatchStatus) -> RepoResult<()>;
    async fn update_result(&self, id: Uuid, winner: Winner) -> RepoResult<()>;
    async fn update_abort(&self, id: Uuid, from: MatchStatus, reason: &str) -> RepoResult<()>;
    async fn update_ports(
        &self,
        id: Uuid,
        bevy_port: Option<i32>,
        ws_port: Option<i32>,
    ) -> RepoResult<()>;
}

pub struct PgMatchRepo {
    pub pool: PgPool,
}

#[async_trait]
impl MatchRepo for PgMatchRepo {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Match>> {
        let sql = format!("SELECT {MATCH_COLS} FROM matches WHERE id = $1");
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(ref r) => Ok(Some(parse_match(r)?)),
            None => Ok(None),
        }
    }

    async fn list_by_owner(&self, owner_id: i32, limit: i64) -> RepoResult<Vec<Match>> {
        let sql = format!(
            "SELECT {MATCH_COLS} FROM matches WHERE owner_id = $1 ORDER BY created_at DESC LIMIT $2"
        );
        let rows = sqlx::query(&sql)
            .bind(owner_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(parse_match).collect()
    }

    async fn list_by_status(&self, status: MatchStatus, limit: i64) -> RepoResult<Vec<Match>> {
        let sql = format!(
            "SELECT {MATCH_COLS} FROM matches WHERE status = $1 ORDER BY created_at DESC LIMIT $2"
        );
        let rows = sqlx::query(&sql)
            .bind(status.as_str())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(parse_match).collect()
    }

    async fn insert(&self, owner_id: i32, input: &MatchInput) -> RepoResult<Match> {
        let id = Uuid::new_v4();
        let sql = format!(
            "INSERT INTO matches (id, form, room_id, owner_id, mode, scenario_id, win_condition, status) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending') RETURNING {MATCH_COLS}"
        );
        let row = sqlx::query(&sql)
            .bind(id)
            .bind(input.form.as_str())
            .bind(input.room_id)
            .bind(owner_id)
            .bind(&input.mode)
            .bind(input.scenario_id)
            .bind(&input.win_condition)
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
        parse_match(&row)
    }

    async fn update_status(&self, id: Uuid, from: MatchStatus, to: MatchStatus) -> RepoResult<()> {
        let sql = match to {
            MatchStatus::Running => {
                "UPDATE matches SET status = $1, started_at = COALESCE(started_at, CURRENT_TIMESTAMP) WHERE id = $2 AND status = $3"
            }
            MatchStatus::Finished | MatchStatus::Aborted => {
                "UPDATE matches SET status = $1, finished_at = CURRENT_TIMESTAMP WHERE id = $2 AND status = $3"
            }
            _ => "UPDATE matches SET status = $1 WHERE id = $2 AND status = $3",
        };
        let result = sqlx::query(sql)
            .bind(to.as_str())
            .bind(id)
            .bind(from.as_str())
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn update_result(&self, id: Uuid, winner: Winner) -> RepoResult<()> {
        let result = sqlx::query("UPDATE matches SET status = 'finished', winner_team = $1, finished_at = CURRENT_TIMESTAMP WHERE id = $2")
            .bind(winner.as_str()).bind(id).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn update_abort(&self, id: Uuid, from: MatchStatus, reason: &str) -> RepoResult<()> {
        let result = sqlx::query("UPDATE matches SET status = 'aborted', abort_reason = $1, finished_at = CURRENT_TIMESTAMP WHERE id = $2 AND status = $3")
            .bind(reason).bind(id).bind(from.as_str()).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn update_ports(
        &self,
        id: Uuid,
        bevy_port: Option<i32>,
        ws_port: Option<i32>,
    ) -> RepoResult<()> {
        let result = sqlx::query("UPDATE matches SET bevy_port = $1, ws_port = $2 WHERE id = $3")
            .bind(bevy_port)
            .bind(ws_port)
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
}

const PART_COLS: &str = "id, match_id, agent_snapshot_id, agent_id, user_id, team, result";

fn parse_participant(r: &sqlx::postgres::PgRow) -> RepoResult<MatchParticipant> {
    let team_str: String = r.try_get("team")?;
    let team = Team::from_str(&team_str)
        .ok_or_else(|| RepoError::Internal(format!("unknown team: {team_str}")))?;
    let result_str: Option<String> = r.try_get("result")?;
    let result = result_str
        .map(|s| {
            ParticipantResult::from_str(&s)
                .ok_or_else(|| RepoError::Internal(format!("unknown participant result: {s}")))
        })
        .transpose()?;
    Ok(MatchParticipant {
        id: r.try_get("id")?,
        match_id: r.try_get("match_id")?,
        agent_snapshot_id: r.try_get("agent_snapshot_id")?,
        agent_id: r.try_get("agent_id")?,
        user_id: r.try_get("user_id")?,
        team,
        result,
    })
}

#[async_trait]
pub trait MatchParticipantRepo: Send + Sync {
    async fn find_by_match(&self, match_id: Uuid) -> RepoResult<Vec<MatchParticipant>>;
    async fn insert(
        &self,
        match_id: Uuid,
        input: &ParticipantInput,
    ) -> RepoResult<MatchParticipant>;
    async fn update_result(
        &self,
        id: Uuid,
        result: ParticipantResult,
        final_stats: Option<serde_json::Value>,
    ) -> RepoResult<()>;
    async fn update_entity_id(&self, id: Uuid, bevy_entity_id: i64) -> RepoResult<()>;
    async fn update_result_by_team(
        &self,
        match_id: Uuid,
        team: Team,
        result: ParticipantResult,
    ) -> RepoResult<u64>;
}

pub struct PgMatchParticipantRepo {
    pub pool: PgPool,
}

#[async_trait]
impl MatchParticipantRepo for PgMatchParticipantRepo {
    async fn find_by_match(&self, match_id: Uuid) -> RepoResult<Vec<MatchParticipant>> {
        let sql = format!(
            "SELECT {PART_COLS} FROM match_participants WHERE match_id = $1 ORDER BY team, id"
        );
        let rows = sqlx::query(&sql)
            .bind(match_id)
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(parse_participant).collect()
    }

    async fn insert(
        &self,
        match_id: Uuid,
        input: &ParticipantInput,
    ) -> RepoResult<MatchParticipant> {
        let id = Uuid::new_v4();
        let sql = format!(
            "INSERT INTO match_participants (id, match_id, agent_snapshot_id, agent_id, user_id, team) VALUES ($1, $2, $3, $4, $5, $6) RETURNING {PART_COLS}"
        );
        let row = sqlx::query(&sql)
            .bind(id)
            .bind(match_id)
            .bind(input.agent_snapshot_id)
            .bind(input.agent_id)
            .bind(input.user_id)
            .bind(input.team.as_str())
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
        parse_participant(&row)
    }

    async fn update_result(
        &self,
        id: Uuid,
        result: ParticipantResult,
        final_stats: Option<serde_json::Value>,
    ) -> RepoResult<()> {
        let r = sqlx::query(
            "UPDATE match_participants SET result = $1, final_stats = $2 WHERE id = $3",
        )
        .bind(result.as_str())
        .bind(&final_stats)
        .bind(id)
        .execute(&self.pool)
        .await?;
        if r.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn update_entity_id(&self, id: Uuid, bevy_entity_id: i64) -> RepoResult<()> {
        let r = sqlx::query("UPDATE match_participants SET bevy_entity_id = $1 WHERE id = $2")
            .bind(bevy_entity_id)
            .bind(id)
            .execute(&self.pool)
            .await?;
        if r.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn update_result_by_team(
        &self,
        match_id: Uuid,
        team: Team,
        result: ParticipantResult,
    ) -> RepoResult<u64> {
        let r = sqlx::query(
            "UPDATE match_participants SET result = $1 WHERE match_id = $2 AND team = $3",
        )
        .bind(result.as_str())
        .bind(match_id)
        .bind(team.as_str())
        .execute(&self.pool)
        .await?;
        Ok(r.rows_affected())
    }
}

const EVENT_COLS: &str = "seq, event_type, agent_id, payload, game_time_ms";

fn parse_event(r: &sqlx::postgres::PgRow) -> RepoResult<MatchEvent> {
    Ok(MatchEvent {
        seq: r.try_get("seq")?,
        event_type: r.try_get("event_type")?,
        agent_id: r.try_get("agent_id")?,
        payload: r.try_get("payload")?,
        game_time_ms: r.try_get("game_time_ms")?,
    })
}

#[async_trait]
pub trait MatchEventRepo: Send + Sync {
    async fn append(&self, match_id: Uuid, event: &MatchEventInput) -> RepoResult<MatchEvent>;
    async fn list_by_match(
        &self,
        match_id: Uuid,
        from_seq: i32,
        limit: i64,
    ) -> RepoResult<Vec<MatchEvent>>;
}

pub struct PgMatchEventRepo {
    pub pool: PgPool,
}

#[async_trait]
impl MatchEventRepo for PgMatchEventRepo {
    async fn append(&self, match_id: Uuid, event: &MatchEventInput) -> RepoResult<MatchEvent> {
        let sql = format!(
            "INSERT INTO match_events (match_id, seq, event_type, agent_id, payload, game_time_ms) \
             SELECT $1, COALESCE(MAX(seq), 0) + 1, $2, $3, $4, $5 FROM match_events WHERE match_id = $1 \
             RETURNING {EVENT_COLS}"
        );
        let row = sqlx::query(&sql)
            .bind(match_id)
            .bind(&event.event_type)
            .bind(event.agent_id)
            .bind(&event.payload)
            .bind(event.game_time_ms)
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
        parse_event(&row)
    }

    async fn list_by_match(
        &self,
        match_id: Uuid,
        from_seq: i32,
        limit: i64,
    ) -> RepoResult<Vec<MatchEvent>> {
        let sql = format!(
            "SELECT {EVENT_COLS} FROM match_events WHERE match_id = $1 AND seq >= $2 ORDER BY seq LIMIT $3"
        );
        let rows = sqlx::query(&sql)
            .bind(match_id)
            .bind(from_seq)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(parse_event).collect()
    }
}
