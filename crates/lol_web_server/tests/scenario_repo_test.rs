//! Scenario 子系统 repository 集成测试。

mod common;

use common::setup_pg;
use lol_web_server::domain::RepoError;
use lol_web_server::domain::scenario::ScenarioInput;
use lol_web_server::repository::scenario_repo::{PgScenarioRepo, ScenarioRepo};
use uuid::Uuid;

async fn create_user(pool: &sqlx::PgPool, phone: &str) -> i32 {
    use sqlx::Row;
    let row = sqlx::query("INSERT INTO users (phone, password_hash) VALUES ($1, $2) RETURNING id")
        .bind(phone)
        .bind("hash")
        .fetch_one(pool)
        .await
        .unwrap();
    row.get("id")
}

fn sample_input(name: &str) -> ScenarioInput {
    ScenarioInput {
        name: name.into(),
        agents: serde_json::json!([{"champion":"Riven"}]),
    }
}

#[tokio::test]
async fn find_missing_returns_none() {
    let fx = setup_pg().await;
    assert!(
        PgScenarioRepo { pool: fx.pool }
            .find_by_id(Uuid::new_v4())
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn insert_find_roundtrip() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600001001").await;
    let repo = PgScenarioRepo {
        pool: fx.pool.clone(),
    };
    let created = repo.insert(owner, &sample_input("激进5v5")).await.unwrap();
    let found = repo.find_by_id(created.id).await.unwrap().unwrap();
    assert_eq!(found.name, "激进5v5");
}

#[tokio::test]
async fn duplicate_name_per_owner_rejected() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600001002").await;
    let repo = PgScenarioRepo {
        pool: fx.pool.clone(),
    };
    repo.insert(owner, &sample_input("dup")).await.unwrap();
    let err = repo.insert(owner, &sample_input("dup")).await.unwrap_err();
    assert!(matches!(err, RepoError::UniqueViolation));
}

#[tokio::test]
async fn win_condition_upsert_and_read() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600001003").await;
    let repo = PgScenarioRepo {
        pool: fx.pool.clone(),
    };
    let s = repo.insert(owner, &sample_input("wc")).await.unwrap();
    assert!(repo.get_win_condition(owner, s.id).await.unwrap().is_none());
    let cond = serde_json::json!({"op":"and","args":[{"type":"kill","threshold":10}]});
    repo.save_win_condition(owner, s.id, &cond).await.unwrap();
    assert_eq!(
        repo.get_win_condition(owner, s.id).await.unwrap().unwrap(),
        cond
    );
    let cond2 = serde_json::json!({"type":"tower","count":3});
    repo.save_win_condition(owner, s.id, &cond2).await.unwrap();
    assert_eq!(
        repo.get_win_condition(owner, s.id).await.unwrap().unwrap(),
        cond2
    );
}

#[tokio::test]
async fn win_condition_cascade_when_scenario_deleted() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600001004").await;
    let repo = PgScenarioRepo {
        pool: fx.pool.clone(),
    };
    let s = repo.insert(owner, &sample_input("cascade")).await.unwrap();
    repo.save_win_condition(owner, s.id, &serde_json::json!({"x":1}))
        .await
        .unwrap();
    repo.delete(s.id).await.unwrap();
    assert!(repo.get_win_condition(owner, s.id).await.unwrap().is_none());
}

#[tokio::test]
async fn scenario_cascade_when_user_deleted() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600001005").await;
    let repo = PgScenarioRepo {
        pool: fx.pool.clone(),
    };
    let s = repo.insert(owner, &sample_input("ucascade")).await.unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(owner)
        .execute(&fx.pool)
        .await
        .unwrap();
    assert!(repo.find_by_id(s.id).await.unwrap().is_none());
}

#[tokio::test]
async fn update_changes_fields() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600001006").await;
    let repo = PgScenarioRepo {
        pool: fx.pool.clone(),
    };
    let s = repo.insert(owner, &sample_input("orig")).await.unwrap();
    repo.update(
        s.id,
        &ScenarioInput {
            name: "updated".into(),
            agents: serde_json::json!([{"champion":"Yasuo"}]),
        },
    )
    .await
    .unwrap();
    let found = repo.find_by_id(s.id).await.unwrap().unwrap();
    assert_eq!(found.name, "updated");
}

#[tokio::test]
async fn delete_removes() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600001007").await;
    let repo = PgScenarioRepo {
        pool: fx.pool.clone(),
    };
    let s = repo.insert(owner, &sample_input("del")).await.unwrap();
    repo.delete(s.id).await.unwrap();
    assert!(repo.find_by_id(s.id).await.unwrap().is_none());
}
