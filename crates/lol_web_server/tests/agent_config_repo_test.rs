//! AgentConfig 子系统 repository 集成测试（testcontainers 真 PG）。

mod common;

use common::setup_pg;
use lol_web_server::domain::RepoError;
use lol_web_server::domain::agent_config::{AgentConfigInput, AgentType};
use lol_web_server::domain::spawn_preset::Visibility;
use lol_web_server::repository::agent_config_repo::{AgentConfigRepo, PgAgentConfigRepo};
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

fn sample_input(name: &str) -> AgentConfigInput {
    AgentConfigInput {
        name: name.into(),
        agent_type: AgentType::Llm,
        prompt: "aggro".into(),
        preamble: "global".into(),
        model: "claude".into(),
        config_json: serde_json::json!({"depth": 2}),
        visibility: Visibility::Private,
    }
}

#[tokio::test]
async fn find_missing_returns_none() {
    let fx = setup_pg().await;
    let repo = PgAgentConfigRepo { pool: fx.pool };
    assert!(repo.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[tokio::test]
async fn insert_find_roundtrip_preserves_jsonb() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13700000001").await;
    let repo = PgAgentConfigRepo {
        pool: fx.pool.clone(),
    };
    let input = AgentConfigInput {
        config_json: serde_json::json!({"tools": ["bash"], "depth": 3}),
        ..sample_input("激进")
    };
    let cfg = repo.insert(owner, &input).await.unwrap();

    let found = repo.find_by_id(cfg.id).await.unwrap().unwrap();
    assert_eq!(found.name, "激进");
    assert_eq!(found.agent_type, AgentType::Llm);
    assert_eq!(found.prompt, "aggro");
    assert_eq!(
        found.config_json,
        serde_json::json!({"tools": ["bash"], "depth": 3})
    );
    assert_eq!(found.visibility, Visibility::Private);
}

#[tokio::test]
async fn list_by_owner() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13700000002").await;
    let other = create_user(&fx.pool, "13700000003").await;
    let repo = PgAgentConfigRepo {
        pool: fx.pool.clone(),
    };
    repo.insert(owner, &sample_input("A")).await.unwrap();
    repo.insert(owner, &sample_input("B")).await.unwrap();
    repo.insert(other, &sample_input("C")).await.unwrap();

    assert_eq!(repo.list_by_owner(owner).await.unwrap().len(), 2);
    assert_eq!(repo.list_by_owner(other).await.unwrap().len(), 1);
}

#[tokio::test]
async fn count_by_owner() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13700000004").await;
    let repo = PgAgentConfigRepo {
        pool: fx.pool.clone(),
    };
    assert_eq!(repo.count_by_owner(owner).await.unwrap(), 0);
    repo.insert(owner, &sample_input("A")).await.unwrap();
    repo.insert(owner, &sample_input("B")).await.unwrap();
    assert_eq!(repo.count_by_owner(owner).await.unwrap(), 2);
}

#[tokio::test]
async fn duplicate_name_per_owner_rejected() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13700000005").await;
    let repo = PgAgentConfigRepo {
        pool: fx.pool.clone(),
    };
    repo.insert(owner, &sample_input("dup")).await.unwrap();
    let err = repo.insert(owner, &sample_input("dup")).await.unwrap_err();
    assert!(matches!(err, RepoError::UniqueViolation));
}

#[tokio::test]
async fn update_changes_fields() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13700000006").await;
    let repo = PgAgentConfigRepo {
        pool: fx.pool.clone(),
    };
    let cfg = repo.insert(owner, &sample_input("orig")).await.unwrap();

    repo.update(
        cfg.id,
        &AgentConfigInput {
            name: "updated".into(),
            agent_type: AgentType::Script,
            prompt: "new prompt".into(),
            preamble: "".into(),
            model: "js".into(),
            config_json: serde_json::json!({"script": "x"}),
            visibility: Visibility::Public,
        },
    )
    .await
    .unwrap();

    let found = repo.find_by_id(cfg.id).await.unwrap().unwrap();
    assert_eq!(found.name, "updated");
    assert_eq!(found.agent_type, AgentType::Script);
    assert_eq!(found.visibility, Visibility::Public);
}

#[tokio::test]
async fn update_missing_returns_not_found() {
    let fx = setup_pg().await;
    let repo = PgAgentConfigRepo { pool: fx.pool };
    let err = repo
        .update(Uuid::new_v4(), &sample_input("x"))
        .await
        .unwrap_err();
    assert!(matches!(err, RepoError::NotFound));
}

#[tokio::test]
async fn delete_removes() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13700000007").await;
    let repo = PgAgentConfigRepo {
        pool: fx.pool.clone(),
    };
    let cfg = repo.insert(owner, &sample_input("del")).await.unwrap();
    repo.delete(cfg.id).await.unwrap();
    assert!(repo.find_by_id(cfg.id).await.unwrap().is_none());
}

#[tokio::test]
async fn delete_blocked_by_foreign_key_when_referenced_by_agent() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13700000008").await;
    let repo = PgAgentConfigRepo {
        pool: fx.pool.clone(),
    };
    let cfg = repo
        .insert(owner, &sample_input("referenced"))
        .await
        .unwrap();

    // 插一个 agent 引用该 config（需先有 spawn_preset）
    let spawn_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO spawn_presets (id, owner_id, name, x, z, team) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(spawn_id)
    .bind(owner)
    .bind("sp")
    .bind(1000.0)
    .bind(1000.0)
    .bind("order")
    .execute(&fx.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO agents (id, owner_id, name, champion, agent_config_id, spawn_preset_id) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::new_v4())
    .bind(owner)
    .bind("Riven")
    .bind("Riven")
    .bind(cfg.id)
    .bind(spawn_id)
    .execute(&fx.pool)
    .await
    .unwrap();

    // 删除被引用的 config 应被 FK RESTRICT 阻止
    let err = repo.delete(cfg.id).await.unwrap_err();
    assert!(matches!(err, RepoError::ForeignKeyViolation));
}

#[tokio::test]
async fn cascade_delete_when_user_removed() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13700000009").await;
    let repo = PgAgentConfigRepo {
        pool: fx.pool.clone(),
    };
    let cfg = repo.insert(owner, &sample_input("orphan")).await.unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(owner)
        .execute(&fx.pool)
        .await
        .unwrap();
    assert!(repo.find_by_id(cfg.id).await.unwrap().is_none());
}
