//! Essence（精粹）+ Subscription（订阅）子系统的持久层。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::essence::{BillingPlan, EssenceTransaction, SubscriptionStatus};
use crate::domain::{RepoError, RepoResult};

#[async_trait]
pub trait EssenceRepo: Send + Sync {
    async fn get_balance(&self, user_id: i32) -> RepoResult<i64>;
    async fn add_transaction(
        &self,
        user_id: i32,
        delta: i64,
        reason: String,
        reference: Option<String>,
    ) -> RepoResult<i64>;
    async fn get_transactions(
        &self,
        user_id: i32,
        limit: i64,
        offset: i64,
    ) -> RepoResult<Vec<EssenceTransaction>>;
    async fn find_by_reference(
        &self,
        user_id: i32,
        reference: &str,
    ) -> RepoResult<Option<EssenceTransaction>>;
}

pub struct PgEssenceRepo {
    pub pool: PgPool,
}

fn parse_tx(r: &sqlx::postgres::PgRow) -> RepoResult<EssenceTransaction> {
    Ok(EssenceTransaction {
        id: r.try_get("id")?,
        user_id: r.try_get("user_id")?,
        delta: r.try_get("delta")?,
        reason: r.try_get("reason")?,
        reference: r.try_get("reference")?,
        balance_after: r.try_get("balance_after")?,
    })
}

const TX_COLS: &str = "id, user_id, delta, reason, reference, balance_after";

#[async_trait]
impl EssenceRepo for PgEssenceRepo {
    async fn get_balance(&self, user_id: i32) -> RepoResult<i64> {
        sqlx::query("INSERT INTO essence_balances (user_id, amount) VALUES ($1, 0) ON CONFLICT (user_id) DO NOTHING")
            .bind(user_id).execute(&self.pool).await?;
        let amount: i64 =
            sqlx::query_scalar("SELECT amount FROM essence_balances WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(amount)
    }

    async fn add_transaction(
        &self,
        user_id: i32,
        delta: i64,
        reason: String,
        reference: Option<String>,
    ) -> RepoResult<i64> {
        let mut tx = self.pool.begin().await?;
        let current: Option<i64> =
            sqlx::query_scalar("SELECT amount FROM essence_balances WHERE user_id = $1 FOR UPDATE")
                .bind(user_id)
                .fetch_optional(&mut *tx)
                .await?;
        let current = match current {
            Some(v) => v,
            None => {
                sqlx::query("INSERT INTO essence_balances (user_id, amount) VALUES ($1, 0)")
                    .bind(user_id)
                    .execute(&mut *tx)
                    .await?;
                0
            }
        };
        let new_balance = current + delta;
        let res = sqlx::query("UPDATE essence_balances SET amount = $1, updated_at = CURRENT_TIMESTAMP WHERE user_id = $2")
            .bind(new_balance).bind(user_id).execute(&mut *tx).await?;
        if res.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        let row = sqlx::query(&format!(
            "INSERT INTO essence_transactions (user_id, delta, reason, reference, balance_after) VALUES ($1, $2, $3, $4, $5) RETURNING {TX_COLS}"
        ))
        .bind(user_id).bind(delta).bind(reason).bind(reference).bind(new_balance)
        .fetch_one(&mut *tx).await?;
        let _ = parse_tx(&row)?;
        tx.commit().await?;
        Ok(new_balance)
    }

    async fn get_transactions(
        &self,
        user_id: i32,
        limit: i64,
        offset: i64,
    ) -> RepoResult<Vec<EssenceTransaction>> {
        let rows = sqlx::query(&format!(
            "SELECT {TX_COLS} FROM essence_transactions WHERE user_id = $1 ORDER BY created_at DESC, id DESC LIMIT $2 OFFSET $3"
        ))
        .bind(user_id).bind(limit).bind(offset).fetch_all(&self.pool).await?;
        rows.iter().map(parse_tx).collect()
    }

    async fn find_by_reference(
        &self,
        user_id: i32,
        reference: &str,
    ) -> RepoResult<Option<EssenceTransaction>> {
        let row = sqlx::query(&format!("SELECT {TX_COLS} FROM essence_transactions WHERE user_id = $1 AND reference = $2 LIMIT 1"))
            .bind(user_id).bind(reference).fetch_optional(&self.pool).await?;
        match row {
            Some(ref r) => Ok(Some(parse_tx(r)?)),
            None => Ok(None),
        }
    }
}

// ── SubscriptionRepo ──

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: i32,
    pub plan_id: String,
    pub status: SubscriptionStatus,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub auto_renew: bool,
}

#[async_trait]
pub trait SubscriptionRepo: Send + Sync {
    async fn get_active(&self, user_id: i32) -> RepoResult<Option<Subscription>>;
    async fn insert(
        &self,
        user_id: i32,
        plan_id: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        auto_renew: bool,
    ) -> RepoResult<Subscription>;
    async fn deactivate(&self, user_id: i32) -> RepoResult<()>;
    async fn get_plan(&self, plan_id: &str) -> RepoResult<Option<BillingPlan>>;
}

pub struct PgSubscriptionRepo {
    pub pool: PgPool,
}

const SUB_COLS: &str = "id, user_id, plan_id, status, period_start, period_end, auto_renew";

fn parse_subscription(r: &sqlx::postgres::PgRow) -> RepoResult<Subscription> {
    let status_str: String = r.try_get("status")?;
    let status = SubscriptionStatus::from_str(&status_str)
        .ok_or_else(|| RepoError::Internal(format!("unknown subscription status: {status_str}")))?;
    Ok(Subscription {
        id: r.try_get("id")?,
        user_id: r.try_get("user_id")?,
        plan_id: r.try_get("plan_id")?,
        status,
        period_start: r.try_get("period_start")?,
        period_end: r.try_get("period_end")?,
        auto_renew: r.try_get("auto_renew")?,
    })
}

#[async_trait]
impl SubscriptionRepo for PgSubscriptionRepo {
    async fn get_active(&self, user_id: i32) -> RepoResult<Option<Subscription>> {
        let row = sqlx::query(&format!("SELECT {SUB_COLS} FROM subscriptions WHERE user_id = $1 AND status = 'active' ORDER BY period_end DESC LIMIT 1"))
            .bind(user_id).fetch_optional(&self.pool).await?;
        match row {
            Some(ref r) => Ok(Some(parse_subscription(r)?)),
            None => Ok(None),
        }
    }

    async fn insert(
        &self,
        user_id: i32,
        plan_id: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        auto_renew: bool,
    ) -> RepoResult<Subscription> {
        let id = Uuid::new_v4();
        let row = sqlx::query(&format!(
            "INSERT INTO subscriptions (id, user_id, plan_id, status, period_start, period_end, auto_renew) VALUES ($1, $2, $3, 'active', $4, $5, $6) RETURNING {SUB_COLS}"
        ))
        .bind(id).bind(user_id).bind(plan_id).bind(period_start).bind(period_end).bind(auto_renew)
        .fetch_one(&self.pool).await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db) = e {
                if db.is_foreign_key_violation() { return RepoError::ForeignKeyViolation; }
            }
            RepoError::Db(e)
        })?;
        parse_subscription(&row)
    }

    async fn deactivate(&self, user_id: i32) -> RepoResult<()> {
        sqlx::query("UPDATE subscriptions SET status = 'cancelled' WHERE user_id = $1 AND status = 'active'")
            .bind(user_id).execute(&self.pool).await?;
        Ok(())
    }

    async fn get_plan(&self, plan_id: &str) -> RepoResult<Option<BillingPlan>> {
        let row = sqlx::query("SELECT id, name, price_cents, essence_per_month, max_agents FROM billing_plans WHERE id = $1")
            .bind(plan_id).fetch_optional(&self.pool).await?;
        match row {
            Some(r) => Ok(Some(BillingPlan {
                id: r.try_get("id")?,
                name: r.try_get("name")?,
                price_cents: r.try_get("price_cents")?,
                essence_per_month: r.try_get("essence_per_month")?,
                max_agents: r.try_get("max_agents")?,
            })),
            None => Ok(None),
        }
    }
}
