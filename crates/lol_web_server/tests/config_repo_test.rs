//! Config 子系统 repository 集成测试（testcontainers 真 PG）。
//!
//! 黄金示例：repo 测试必须碰真 DB，验证 SQL 正确性、约束触发。

mod common;

use common::setup_pg;
use lol_web_server::domain::config::AiConfig;
use lol_web_server::repository::config_repo::{ConfigRepo, PgConfigRepo};

#[tokio::test]
async fn find_by_user_returns_none_when_not_set() {
    let fx = setup_pg().await;
    let user_id = create_user(&fx.pool, "13800000010").await;
    let repo = PgConfigRepo {
        pool: fx.pool.clone(),
    };
    let result = repo.find_by_user(user_id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn upsert_then_find_roundtrip() {
    let fx = setup_pg().await;
    let user_id = create_user(&fx.pool, "13800000011").await;
    let repo = PgConfigRepo {
        pool: fx.pool.clone(),
    };

    let cfg = AiConfig {
        api_key: "sk-roundtrip".into(),
        base_url: "https://api.round.com".into(),
        preamble: "be aggressive".into(),
    };
    repo.upsert(user_id, &cfg).await.unwrap();

    let found = repo.find_by_user(user_id).await.unwrap().unwrap();
    assert_eq!(found, cfg);
}

#[tokio::test]
async fn upsert_is_idempotent_update() {
    let fx = setup_pg().await;
    let user_id = create_user(&fx.pool, "13800000012").await;
    let repo = PgConfigRepo {
        pool: fx.pool.clone(),
    };

    // 第一次写
    repo.upsert(
        user_id,
        &AiConfig {
            api_key: "first".into(),
            ..AiConfig::empty()
        },
    )
    .await
    .unwrap();

    // 第二次覆盖
    repo.upsert(
        user_id,
        &AiConfig {
            api_key: "second".into(),
            ..AiConfig::empty()
        },
    )
    .await
    .unwrap();

    let found = repo.find_by_user(user_id).await.unwrap().unwrap();
    assert_eq!(found.api_key, "second");
}

#[tokio::test]
async fn configs_isolated_per_user() {
    let fx = setup_pg().await;
    let user_a = create_user(&fx.pool, "13800000013").await;
    let user_b = create_user(&fx.pool, "13800000014").await;
    let repo = PgConfigRepo {
        pool: fx.pool.clone(),
    };

    repo.upsert(
        user_a,
        &AiConfig {
            api_key: "a-key".into(),
            ..AiConfig::empty()
        },
    )
    .await
    .unwrap();
    repo.upsert(
        user_b,
        &AiConfig {
            api_key: "b-key".into(),
            ..AiConfig::empty()
        },
    )
    .await
    .unwrap();

    let cfg_a = repo.find_by_user(user_a).await.unwrap().unwrap();
    let cfg_b = repo.find_by_user(user_b).await.unwrap().unwrap();
    assert_eq!(cfg_a.api_key, "a-key");
    assert_eq!(cfg_b.api_key, "b-key");
}

#[tokio::test]
async fn cascade_delete_when_user_removed() {
    let fx = setup_pg().await;
    let user_id = create_user(&fx.pool, "13800000015").await;
    let repo = PgConfigRepo {
        pool: fx.pool.clone(),
    };

    repo.upsert(user_id, &AiConfig::empty()).await.unwrap();
    assert!(repo.find_by_user(user_id).await.unwrap().is_some());

    // 删 user，config 应级联删除
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&fx.pool)
        .await
        .unwrap();

    assert!(repo.find_by_user(user_id).await.unwrap().is_none());
}

/// 插入一个测试用户，返回其 id。
async fn create_user(pool: &sqlx::PgPool, phone: &str) -> i32 {
    let row = sqlx::query("INSERT INTO users (phone, password_hash) VALUES ($1, $2) RETURNING id")
        .bind(phone)
        .bind("hash")
        .fetch_one(pool)
        .await
        .unwrap();
    use sqlx::Row;
    row.get("id")
}
