//! SpawnPreset 子系统的持久层。

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::spawn_preset::{SpawnPreset, SpawnPresetInput, Team, Visibility};
use crate::domain::{RepoError, RepoResult};

#[async_trait]
pub trait SpawnPresetRepo: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<SpawnPreset>>;
    async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<SpawnPreset>>;
    async fn insert(&self, owner_id: i32, input: &SpawnPresetInput) -> RepoResult<SpawnPreset>;
    async fn update(&self, id: Uuid, input: &SpawnPresetInput) -> RepoResult<()>;
    async fn delete(&self, id: Uuid) -> RepoResult<()>;
}

pub struct PgSpawnPresetRepo {
    pub pool: PgPool,
}

fn parse_row(r: &sqlx::postgres::PgRow) -> RepoResult<SpawnPreset> {
    let team_str: String = r.try_get("team")?;
    let vis_str: String = r.try_get("visibility")?;
    let team = Team::from_str(&team_str)
        .ok_or_else(|| RepoError::Internal(format!("unknown team: {team_str}")))?;
    let visibility = Visibility::from_str(&vis_str)
        .ok_or_else(|| RepoError::Internal(format!("unknown visibility: {vis_str}")))?;
    Ok(SpawnPreset {
        id: r.try_get("id")?,
        owner_id: r.try_get("owner_id")?,
        name: r.try_get("name")?,
        x: r.try_get("x")?,
        z: r.try_get("z")?,
        team,
        visibility,
    })
}

#[async_trait]
impl SpawnPresetRepo for PgSpawnPresetRepo {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<SpawnPreset>> {
        let row = sqlx::query(
            "SELECT id, owner_id, name, x, z, team, visibility FROM spawn_presets WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some(ref r) => Ok(Some(parse_row(r)?)),
            None => Ok(None),
        }
    }

    async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<SpawnPreset>> {
        let rows = sqlx::query(
            "SELECT id, owner_id, name, x, z, team, visibility \
             FROM spawn_presets WHERE owner_id = $1 ORDER BY name",
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(parse_row).collect()
    }

    async fn insert(&self, owner_id: i32, input: &SpawnPresetInput) -> RepoResult<SpawnPreset> {
        let id = Uuid::new_v4();
        let row = sqlx::query(
            "INSERT INTO spawn_presets (id, owner_id, name, x, z, team, visibility) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             RETURNING id, owner_id, name, x, z, team, visibility",
        )
        .bind(id)
        .bind(owner_id)
        .bind(&input.name)
        .bind(input.x)
        .bind(input.z)
        .bind(input.team.as_str())
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

    async fn update(&self, id: Uuid, input: &SpawnPresetInput) -> RepoResult<()> {
        let result = sqlx::query(
            "UPDATE spawn_presets SET name = $1, x = $2, z = $3, team = $4, visibility = $5 \
             WHERE id = $6",
        )
        .bind(&input.name)
        .bind(input.x)
        .bind(input.z)
        .bind(input.team.as_str())
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
        let result = sqlx::query("DELETE FROM spawn_presets WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
}
