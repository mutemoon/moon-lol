//! ModelProvider 子系统的持久层（model_providers 表）。

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::model_provider::{ModelProvider, ModelProviderInput};
use crate::domain::{RepoError, RepoResult};

#[async_trait]
pub trait ModelProviderRepo: Send + Sync {
    async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<ModelProvider>>;
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<ModelProvider>>;
    /// 运行时解析：按 id + owner 校验归属后返回含明文密钥的记录。
    async fn find_for_runtime(&self, id: Uuid, owner_id: i32) -> RepoResult<Option<ModelProvider>>;
    async fn insert(&self, owner_id: i32, input: &ModelProviderInput) -> RepoResult<ModelProvider>;
    /// 更新；api_key 空串时保留旧值（COALESCE(NULLIF(...,''), api_key)）。
    async fn update(&self, id: Uuid, input: &ModelProviderInput) -> RepoResult<()>;
    async fn delete(&self, id: Uuid) -> RepoResult<()>;
}

pub struct PgModelProviderRepo {
    pub pool: PgPool,
}

const SELECT_COLS: &str = "id, owner_id, name, category, preset_type, base_url, api_key, \
     api_format, models, enabled, website_url, api_key_url, icon, icon_color, sort_order";

fn parse_row(r: &sqlx::postgres::PgRow) -> RepoResult<ModelProvider> {
    let models_val: serde_json::Value = r.try_get("models")?;
    let models: Vec<lol_agent_runtime::ModelConfig> = serde_json::from_value(models_val)
        .map_err(|e| RepoError::Internal(format!("models JSON 解析失败: {e}")))?;
    Ok(ModelProvider {
        id: r.try_get("id")?,
        owner_id: r.try_get("owner_id")?,
        name: r.try_get("name")?,
        category: r.try_get("category")?,
        preset_type: r.try_get("preset_type")?,
        base_url: r.try_get("base_url")?,
        api_key: r.try_get("api_key")?,
        api_format: r.try_get("api_format")?,
        models,
        enabled: r.try_get("enabled")?,
        website_url: r.try_get("website_url")?,
        api_key_url: r.try_get("api_key_url")?,
        icon: r.try_get("icon")?,
        icon_color: r.try_get("icon_color")?,
        sort_order: r.try_get("sort_order")?,
    })
}

fn map_db_err(e: sqlx::Error) -> RepoError {
    if let sqlx::Error::Database(ref db) = e {
        if db.is_unique_violation() {
            return RepoError::UniqueViolation;
        }
    }
    RepoError::Db(e)
}

#[async_trait]
impl ModelProviderRepo for PgModelProviderRepo {
    async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<ModelProvider>> {
        let sql = format!(
            "SELECT {SELECT_COLS} FROM model_providers WHERE owner_id = $1 ORDER BY sort_order, name"
        );
        let rows = sqlx::query(&sql)
            .bind(owner_id)
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(parse_row).collect()
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<ModelProvider>> {
        let sql = format!("SELECT {SELECT_COLS} FROM model_providers WHERE id = $1");
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(ref r) => Ok(Some(parse_row(r)?)),
            None => Ok(None),
        }
    }

    async fn find_for_runtime(&self, id: Uuid, owner_id: i32) -> RepoResult<Option<ModelProvider>> {
        let sql =
            format!("SELECT {SELECT_COLS} FROM model_providers WHERE id = $1 AND owner_id = $2");
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

    async fn insert(&self, owner_id: i32, input: &ModelProviderInput) -> RepoResult<ModelProvider> {
        let id = Uuid::new_v4();
        let sql = format!(
            "INSERT INTO model_providers (id, owner_id, name, category, preset_type, base_url, \
             api_key, api_format, models, enabled, website_url, api_key_url, icon, icon_color, sort_order) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) \
             RETURNING {SELECT_COLS}"
        );
        let row = sqlx::query(&sql)
            .bind(id)
            .bind(owner_id)
            .bind(&input.name)
            .bind(&input.category)
            .bind(&input.preset_type)
            .bind(&input.base_url)
            .bind(&input.api_key)
            .bind(&input.api_format)
            .bind(serde_json::to_value(&input.models).unwrap_or(serde_json::Value::Array(vec![])))
            .bind(input.enabled)
            .bind(&input.website_url)
            .bind(&input.api_key_url)
            .bind(&input.icon)
            .bind(&input.icon_color)
            .bind(input.sort_order)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| map_db_err(e))?;
        parse_row(&row)
    }

    async fn update(&self, id: Uuid, input: &ModelProviderInput) -> RepoResult<()> {
        // api_key 空串 → 保留旧值。
        let res = sqlx::query(
            "UPDATE model_providers SET name = $1, category = $2, preset_type = $3, base_url = $4, \
             api_key = COALESCE(NULLIF($5, ''), api_key), api_format = $6, models = $7, \
             enabled = $8, website_url = $9, api_key_url = $10, icon = $11, icon_color = $12, \
             sort_order = $13, updated_at = CURRENT_TIMESTAMP WHERE id = $14",
        )
        .bind(&input.name)
        .bind(&input.category)
        .bind(&input.preset_type)
        .bind(&input.base_url)
        .bind(&input.api_key)
        .bind(&input.api_format)
        .bind(serde_json::to_value(&input.models).unwrap_or(serde_json::Value::Array(vec![])))
        .bind(input.enabled)
        .bind(&input.website_url)
        .bind(&input.api_key_url)
        .bind(&input.icon)
        .bind(&input.icon_color)
        .bind(input.sort_order)
        .bind(id)
        .execute(&self.pool)
        .await?;
        if res.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let res = sqlx::query("DELETE FROM model_providers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
}
