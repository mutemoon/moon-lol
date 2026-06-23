//! Essence + Subscription 子系统 repository 集成测试。

mod common;

use chrono::{Duration, Utc};
use common::setup_pg;
use lol_web_server::domain::ServiceError;
use lol_web_server::domain::essence::{BillingPlan, DAILY_CHECKIN_REWARD};
use lol_web_server::repository::essence_repo::{
    EssenceRepo, PgEssenceRepo, PgSubscriptionRepo, SubscriptionRepo,
};
use lol_web_server::service::essence_service::{
    EssenceService, EssenceServiceImpl, SubscriptionService, SubscriptionServiceImpl,
};
use sqlx::Row;
use std::sync::Arc;

async fn create_user(pool: &sqlx::PgPool, phone: &str) -> i32 {
    let row = sqlx::query("INSERT INTO users (phone, password_hash) VALUES ($1, $2) RETURNING id")
        .bind(phone)
        .bind("hash")
        .fetch_one(pool)
        .await
        .unwrap();
    row.get("id")
}

async fn seed_plans(pool: &sqlx::PgPool) {
    for (i, plan) in BillingPlan::all().iter().enumerate() {
        sqlx::query("INSERT INTO billing_plans (id, name, price_cents, essence_per_month, max_agents, sort_order) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (id) DO NOTHING")
            .bind(&plan.id).bind(&plan.name).bind(plan.price_cents).bind(plan.essence_per_month).bind(plan.max_agents).bind(i as i32)
            .execute(pool).await.unwrap();
    }
}

#[tokio::test]
async fn balance_creates_row_if_missing() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600001001").await;
    let repo = PgEssenceRepo {
        pool: fx.pool.clone(),
    };
    assert_eq!(repo.get_balance(owner).await.unwrap(), 0);
    assert_eq!(repo.get_balance(owner).await.unwrap(), 0);
}

#[tokio::test]
async fn add_transaction_updates_balance() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600001002").await;
    let repo = PgEssenceRepo {
        pool: fx.pool.clone(),
    };
    assert_eq!(
        repo.add_transaction(owner, 500, "recharge".into(), Some("rc_1".into()))
            .await
            .unwrap(),
        500
    );
    assert_eq!(
        repo.add_transaction(owner, -200, "token_deduction".into(), None)
            .await
            .unwrap(),
        300
    );
    let txs = repo.get_transactions(owner, 10, 0).await.unwrap();
    assert_eq!(txs.len(), 2);
    assert_eq!(txs[0].balance_after, 300);
}

#[tokio::test]
async fn find_by_reference() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600001003").await;
    let repo = PgEssenceRepo {
        pool: fx.pool.clone(),
    };
    repo.add_transaction(
        owner,
        100,
        "checkin".into(),
        Some("checkin_2026-06-23".into()),
    )
    .await
    .unwrap();
    assert!(
        repo.find_by_reference(owner, "checkin_2026-06-23")
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        repo.find_by_reference(owner, "checkin_2099-01-01")
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn concurrent_deducts_no_lost_update() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600001004").await;
    let repo = PgEssenceRepo {
        pool: fx.pool.clone(),
    };
    repo.add_transaction(owner, 300, "init".into(), Some("init".into()))
        .await
        .unwrap();
    let pool = fx.pool.clone();
    let mut handles = Vec::new();
    for _ in 0..5 {
        let p = pool.clone();
        handles.push(tokio::spawn(async move {
            PgEssenceRepo { pool: p }
                .add_transaction(owner, -100, "deduct".into(), None)
                .await
        }));
    }
    for h in handles {
        h.await.unwrap().unwrap();
    }
    let final_balance: i64 =
        sqlx::query_scalar("SELECT amount FROM essence_balances WHERE user_id = $1")
            .bind(owner)
            .fetch_one(&fx.pool)
            .await
            .unwrap();
    assert_eq!(final_balance, -200, "并发扣款必须无丢失更新");
}

// ── Subscription ──

#[tokio::test]
async fn plan_lookup() {
    let fx = setup_pg().await;
    seed_plans(&fx.pool).await;
    let repo = PgSubscriptionRepo {
        pool: fx.pool.clone(),
    };
    assert_eq!(
        repo.get_plan("free").await.unwrap().unwrap(),
        BillingPlan::free()
    );
    assert_eq!(repo.get_plan("pro").await.unwrap().unwrap().max_agents, 20);
    assert!(repo.get_plan("ghost").await.unwrap().is_none());
}

#[tokio::test]
async fn subscribe_and_get_active() {
    let fx = setup_pg().await;
    seed_plans(&fx.pool).await;
    let owner = create_user(&fx.pool, "13600001005").await;
    let repo = PgSubscriptionRepo {
        pool: fx.pool.clone(),
    };
    let now = Utc::now();
    let sub = repo
        .insert(owner, "pro", now, now + Duration::days(30), false)
        .await
        .unwrap();
    assert_eq!(sub.plan_id, "pro");
    let active = repo.get_active(owner).await.unwrap().unwrap();
    assert_eq!(active.id, sub.id);
}

#[tokio::test]
async fn deactivate_cancels_active() {
    let fx = setup_pg().await;
    seed_plans(&fx.pool).await;
    let owner = create_user(&fx.pool, "13600001006").await;
    let repo = PgSubscriptionRepo {
        pool: fx.pool.clone(),
    };
    let now = Utc::now();
    repo.insert(owner, "pro", now, now + Duration::days(30), false)
        .await
        .unwrap();
    repo.deactivate(owner).await.unwrap();
    assert!(repo.get_active(owner).await.unwrap().is_none());
}

#[tokio::test]
async fn insert_unknown_plan_fk_violation() {
    let fx = setup_pg().await;
    seed_plans(&fx.pool).await;
    let owner = create_user(&fx.pool, "13600001007").await;
    let repo = PgSubscriptionRepo {
        pool: fx.pool.clone(),
    };
    let now = Utc::now();
    let err = repo
        .insert(owner, "ghost", now, now + Duration::days(30), false)
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        lol_web_server::domain::RepoError::ForeignKeyViolation
    ));
}

// ── 端到端 service ──

#[tokio::test]
async fn checkin_idempotent_e2e() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600001008").await;
    let svc = EssenceServiceImpl::new(Arc::new(PgEssenceRepo {
        pool: fx.pool.clone(),
    }));
    let r1 = svc.check_in(owner, "2026-06-23").await.unwrap();
    assert!(!r1.already_checked_in);
    assert_eq!(r1.balance, DAILY_CHECKIN_REWARD);
    let r2 = svc.check_in(owner, "2026-06-23").await.unwrap();
    assert!(r2.already_checked_in);
    assert_eq!(r2.granted, 0);
}

#[tokio::test]
async fn deduct_insufficient_e2e() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600001009").await;
    let svc = EssenceServiceImpl::new(Arc::new(PgEssenceRepo {
        pool: fx.pool.clone(),
    }));
    svc.grant(owner, 100, "recharge").await.unwrap();
    let err = svc.deduct(owner, 300, "token_deduction").await.unwrap_err();
    assert!(matches!(err, ServiceError::InsufficientEssence { .. }));
    assert_eq!(svc.get_balance(owner).await.unwrap(), 100);
}

#[tokio::test]
async fn agent_limit_e2e() {
    let fx = setup_pg().await;
    seed_plans(&fx.pool).await;
    let owner = create_user(&fx.pool, "13600001010").await;
    let svc = SubscriptionServiceImpl::new(Arc::new(PgSubscriptionRepo {
        pool: fx.pool.clone(),
    }));
    assert_eq!(
        SubscriptionService::get_agent_limit(&svc, owner)
            .await
            .unwrap(),
        5
    );
    svc.subscribe(owner, "pro").await.unwrap();
    assert_eq!(
        SubscriptionService::get_agent_limit(&svc, owner)
            .await
            .unwrap(),
        20
    );
    svc.deactivate(owner).await.unwrap();
    assert_eq!(
        SubscriptionService::get_agent_limit(&svc, owner)
            .await
            .unwrap(),
        5
    );
}
