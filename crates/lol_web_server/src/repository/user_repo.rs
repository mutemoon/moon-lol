//! User/Auth 子系统的持久层（users 表）。

use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::domain::auth::User;
use crate::domain::{RepoError, RepoResult};

#[async_trait]
pub trait UserRepo: Send + Sync {
    /// 按 phone 查找用户。
    async fn find_by_phone(&self, phone: &str) -> RepoResult<Option<(User, String)>>;

    /// 按 id 查找用户。
    async fn find_by_id(&self, id: i32) -> RepoResult<Option<User>>;

    /// 创建用户，返回 (User, id)。
    async fn insert(&self, phone: &str, password_hash: &str) -> RepoResult<User>;

    /// 更新密码哈希。
    async fn update_password(&self, id: i32, password_hash: &str) -> RepoResult<()>;
}

pub struct PgUserRepo {
    pub pool: PgPool,
}

#[async_trait]
impl UserRepo for PgUserRepo {
    async fn find_by_phone(&self, phone: &str) -> RepoResult<Option<(User, String)>> {
        let row = sqlx::query("SELECT id, phone, password_hash FROM users WHERE phone = $1")
            .bind(phone)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(r) => Ok(Some((
                User {
                    id: r.try_get("id")?,
                    phone: r.try_get("phone")?,
                },
                r.try_get("password_hash")?,
            ))),
            None => Ok(None),
        }
    }

    async fn find_by_id(&self, id: i32) -> RepoResult<Option<User>> {
        let row = sqlx::query("SELECT id, phone FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(r) => Ok(Some(User {
                id: r.try_get("id")?,
                phone: r.try_get("phone")?,
            })),
            None => Ok(None),
        }
    }

    async fn insert(&self, phone: &str, password_hash: &str) -> RepoResult<User> {
        let row = sqlx::query(
            "INSERT INTO users (phone, password_hash) VALUES ($1, $2) RETURNING id, phone",
        )
        .bind(phone)
        .bind(password_hash)
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
        Ok(User {
            id: row.try_get("id")?,
            phone: row.try_get("phone")?,
        })
    }

    async fn update_password(&self, id: i32, password_hash: &str) -> RepoResult<()> {
        let result = sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
            .bind(password_hash)
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
}
