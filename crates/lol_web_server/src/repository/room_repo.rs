//! Room 子系统的持久层（rooms + room_members + room_agent_slots）。

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::room::{
    Room, RoomAgentSlot, RoomConstraints, RoomStatus, TeamPolicy, generate_invite_code,
};
use crate::domain::spawn_preset::Team;
use crate::domain::{RepoError, RepoResult};

/// 房间创建输入。
pub struct CreateRoomInput {
    pub name: String,
    pub constraints: RoomConstraints,
}

#[async_trait]
pub trait RoomRepo: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Room>>;
    async fn find_by_invite_code(&self, code: &str) -> RepoResult<Option<Room>>;
    async fn list_by_member(&self, user_id: i32) -> RepoResult<Vec<Room>>;
    async fn list_lobby(&self) -> RepoResult<Vec<Room>>;
    async fn insert(&self, owner_id: i32, input: &CreateRoomInput) -> RepoResult<Room>;
    async fn update_status(&self, id: Uuid, status: RoomStatus) -> RepoResult<()>;
    async fn update_constraints(&self, id: Uuid, constraints: RoomConstraints) -> RepoResult<()>;
    async fn delete(&self, id: Uuid) -> RepoResult<()>;

    // 成员
    async fn count_members(&self, room_id: Uuid) -> RepoResult<i64>;
    async fn add_member(&self, room_id: Uuid, user_id: i32) -> RepoResult<()>;
    async fn remove_member(&self, room_id: Uuid, user_id: i32) -> RepoResult<()>;
    async fn is_member(&self, room_id: Uuid, user_id: i32) -> RepoResult<bool>;

    // 槽位
    async fn list_slots(&self, room_id: Uuid) -> RepoResult<Vec<RoomAgentSlot>>;
    async fn count_slots_by_member(&self, room_id: Uuid, user_id: i32) -> RepoResult<i64>;
    async fn member_existing_team(&self, room_id: Uuid, user_id: i32) -> RepoResult<Option<Team>>;
    async fn add_slot(
        &self,
        room_id: Uuid,
        user_id: i32,
        agent_id: Uuid,
        team: Team,
    ) -> RepoResult<RoomAgentSlot>;
    async fn remove_slot(&self, slot_id: Uuid) -> RepoResult<()>;
}

pub struct PgRoomRepo {
    pub pool: PgPool,
}

fn parse_room(r: &sqlx::postgres::PgRow) -> RepoResult<Room> {
    let status_str: String = r.try_get("status")?;
    let policy_str: String = r.try_get("team_policy")?;
    let status = RoomStatus::from_str(&status_str)
        .ok_or_else(|| RepoError::Internal(format!("unknown status: {status_str}")))?;
    let team_policy = TeamPolicy::from_str(&policy_str)
        .ok_or_else(|| RepoError::Internal(format!("unknown team_policy: {policy_str}")))?;
    Ok(Room {
        id: r.try_get("id")?,
        owner_id: r.try_get("owner_id")?,
        name: r.try_get("name")?,
        invite_code: r.try_get("invite_code")?,
        constraints: RoomConstraints {
            max_members: r.try_get("max_members")?,
            max_agents_per_member: r.try_get("max_agents_per_member")?,
            team_policy,
            lobby_visible: r.try_get("lobby_visible")?,
            prompt_visible: r.try_get("prompt_visible")?,
        },
        status,
    })
}

#[async_trait]
impl RoomRepo for PgRoomRepo {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Room>> {
        let row = sqlx::query(
            "SELECT id, owner_id, name, invite_code, max_members, max_agents_per_member, \
             team_policy, lobby_visible, prompt_visible, status \
             FROM rooms WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(ref r) => Ok(Some(parse_room(r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_invite_code(&self, code: &str) -> RepoResult<Option<Room>> {
        let row = sqlx::query(
            "SELECT id, owner_id, name, invite_code, max_members, max_agents_per_member, \
             team_policy, lobby_visible, prompt_visible, status \
             FROM rooms WHERE invite_code = $1",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(ref r) => Ok(Some(parse_room(r)?)),
            None => Ok(None),
        }
    }

    async fn list_by_member(&self, user_id: i32) -> RepoResult<Vec<Room>> {
        let rows = sqlx::query(
            "SELECT r.id, r.owner_id, r.name, r.invite_code, r.max_members, \
             r.max_agents_per_member, r.team_policy, r.lobby_visible, r.prompt_visible, r.status \
             FROM rooms r JOIN room_members m ON r.id = m.room_id \
             WHERE m.user_id = $1 ORDER BY r.created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(parse_room).collect()
    }

    async fn list_lobby(&self) -> RepoResult<Vec<Room>> {
        let rows = sqlx::query(
            "SELECT id, owner_id, name, invite_code, max_members, max_agents_per_member, \
             team_policy, lobby_visible, prompt_visible, status \
             FROM rooms WHERE lobby_visible = TRUE AND status = 'lobby' \
             ORDER BY created_at DESC LIMIT 50",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(parse_room).collect()
    }

    async fn insert(&self, owner_id: i32, input: &CreateRoomInput) -> RepoResult<Room> {
        let id = Uuid::new_v4();
        let invite_code = generate_invite_code();
        let row = sqlx::query(
            "INSERT INTO rooms (id, owner_id, name, invite_code, max_members, \
             max_agents_per_member, team_policy, lobby_visible, prompt_visible, status) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'lobby') \
             RETURNING id, owner_id, name, invite_code, max_members, max_agents_per_member, \
             team_policy, lobby_visible, prompt_visible, status",
        )
        .bind(id)
        .bind(owner_id)
        .bind(&input.name)
        .bind(&invite_code)
        .bind(input.constraints.max_members)
        .bind(input.constraints.max_agents_per_member)
        .bind(input.constraints.team_policy.as_str())
        .bind(input.constraints.lobby_visible)
        .bind(input.constraints.prompt_visible)
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
        // 房主自动加入成员
        sqlx::query("INSERT INTO room_members (room_id, user_id) VALUES ($1, $2)")
            .bind(id)
            .bind(owner_id)
            .execute(&self.pool)
            .await?;
        parse_room(&row)
    }

    async fn update_status(&self, id: Uuid, status: RoomStatus) -> RepoResult<()> {
        let result = sqlx::query("UPDATE rooms SET status = $1 WHERE id = $2")
            .bind(status.as_str())
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn update_constraints(&self, id: Uuid, constraints: RoomConstraints) -> RepoResult<()> {
        let result = sqlx::query(
            "UPDATE rooms SET max_members = $1, max_agents_per_member = $2, \
             team_policy = $3, lobby_visible = $4, prompt_visible = $5 WHERE id = $6",
        )
        .bind(constraints.max_members)
        .bind(constraints.max_agents_per_member)
        .bind(constraints.team_policy.as_str())
        .bind(constraints.lobby_visible)
        .bind(constraints.prompt_visible)
        .bind(id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM rooms WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn count_members(&self, room_id: Uuid) -> RepoResult<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM room_members WHERE room_id = $1")
            .bind(room_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    async fn add_member(&self, room_id: Uuid, user_id: i32) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO room_members (room_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(room_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn remove_member(&self, room_id: Uuid, user_id: i32) -> RepoResult<()> {
        sqlx::query("DELETE FROM room_members WHERE room_id = $1 AND user_id = $2")
            .bind(room_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn is_member(&self, room_id: Uuid, user_id: i32) -> RepoResult<bool> {
        let row = sqlx::query("SELECT 1 FROM room_members WHERE room_id = $1 AND user_id = $2")
            .bind(room_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.is_some())
    }

    async fn list_slots(&self, room_id: Uuid) -> RepoResult<Vec<RoomAgentSlot>> {
        let rows = sqlx::query(
            "SELECT id, room_id, user_id, agent_id, team FROM room_agent_slots WHERE room_id = $1",
        )
        .bind(room_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter()
            .map(|r| {
                let team_str: String = r.try_get("team")?;
                let team = Team::from_str(&team_str)
                    .ok_or_else(|| RepoError::Internal(format!("unknown team: {team_str}")))?;
                Ok(RoomAgentSlot {
                    id: r.try_get("id")?,
                    room_id: r.try_get("room_id")?,
                    user_id: r.try_get("user_id")?,
                    agent_id: r.try_get("agent_id")?,
                    team,
                })
            })
            .collect()
    }

    async fn count_slots_by_member(&self, room_id: Uuid, user_id: i32) -> RepoResult<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM room_agent_slots WHERE room_id = $1 AND user_id = $2",
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn member_existing_team(&self, room_id: Uuid, user_id: i32) -> RepoResult<Option<Team>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT team FROM room_agent_slots WHERE room_id = $1 AND user_id = $2 LIMIT 1",
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.and_then(|(s,)| Team::from_str(&s)))
    }

    async fn add_slot(
        &self,
        room_id: Uuid,
        user_id: i32,
        agent_id: Uuid,
        team: Team,
    ) -> RepoResult<RoomAgentSlot> {
        let id = Uuid::new_v4();
        let row = sqlx::query(
            "INSERT INTO room_agent_slots (id, room_id, user_id, agent_id, team) \
             VALUES ($1, $2, $3, $4, $5) RETURNING id, room_id, user_id, agent_id, team",
        )
        .bind(id)
        .bind(room_id)
        .bind(user_id)
        .bind(agent_id)
        .bind(team.as_str())
        .fetch_one(&self.pool)
        .await?;
        let team_str: String = row.try_get("team")?;
        let parsed_team = Team::from_str(&team_str)
            .ok_or_else(|| RepoError::Internal(format!("unknown team: {team_str}")))?;
        Ok(RoomAgentSlot {
            id: row.try_get("id")?,
            room_id: row.try_get("room_id")?,
            user_id: row.try_get("user_id")?,
            agent_id: row.try_get("agent_id")?,
            team: parsed_team,
        })
    }

    async fn remove_slot(&self, slot_id: Uuid) -> RepoResult<()> {
        let result = sqlx::query("DELETE FROM room_agent_slots WHERE id = $1")
            .bind(slot_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
}
