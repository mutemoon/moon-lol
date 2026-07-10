//! AgentSnapshot 子系统 repository 集成测试。

mod common;

use common::setup_pg;
use lol_web_server::domain::RepoError;
use lol_web_server::domain::agent::{AgentInput, AgentType};
use lol_web_server::domain::spawn_preset::Visibility;
use lol_web_server::repository::agent_repo::{AgentRepo, PgAgentRepo};
use lol_web_server::repository::agent_snapshot_repo::{AgentSnapshotRepo, PgAgentSnapshotRepo};
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

async fn create_full_agent(
    pool: &sqlx::PgPool,
    owner: i32,
    name: &str,
) -> lol_web_server::domain::agent::Agent {
    PgAgentRepo { pool: pool.clone() }
        .insert(
            owner,
            &AgentInput {
                name: name.into(),
                champion: "Riven".into(),
                agent_type: AgentType::Llm,
                prompt: "prompt".into(),
                model: "model".into(),
                config_json: serde_json::json!({}),
                visibility: Visibility::Private,
            },
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn find_missing_returns_none() {
    let fx = setup_pg().await;
    assert!(
        PgAgentSnapshotRepo { pool: fx.pool }
            .find_by_id(Uuid::new_v4())
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn insert_find_roundtrip() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13700000001").await;
    let agent = create_full_agent(&fx.pool, owner, "锐雯").await;
    let freeze = serde_json::json!({"champion": "Riven", "k": 1});
    let repo = PgAgentSnapshotRepo {
        pool: fx.pool.clone(),
    };
    let snap = repo.insert(agent.id, 1, &freeze).await.unwrap();
    assert_eq!(snap.agent_id, agent.id);
    assert_eq!(snap.version, 1);
    assert_eq!(snap.config_freeze, freeze);
}

#[tokio::test]
async fn version_increments_via_max_version() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13700000002").await;
    let agent = create_full_agent(&fx.pool, owner, "A").await;
    let repo = PgAgentSnapshotRepo {
        pool: fx.pool.clone(),
    };
    assert_eq!(repo.max_version(agent.id).await.unwrap(), None);
    repo.insert(agent.id, 1, &serde_json::json!({"v":1}))
        .await
        .unwrap();
    assert_eq!(repo.max_version(agent.id).await.unwrap(), Some(1));
    let s2 = repo
        .insert(agent.id, 2, &serde_json::json!({"v":2}))
        .await
        .unwrap();
    let latest = repo.find_latest(agent.id).await.unwrap().unwrap();
    assert_eq!(latest.id, s2.id);
    let list = repo.list_by_agent(agent.id).await.unwrap();
    assert_eq!(list.len(), 2);
}

#[tokio::test]
async fn duplicate_agent_version_rejected() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13700000003").await;
    let agent = create_full_agent(&fx.pool, owner, "dup").await;
    let repo = PgAgentSnapshotRepo {
        pool: fx.pool.clone(),
    };
    repo.insert(agent.id, 1, &serde_json::json!({}))
        .await
        .unwrap();
    let err = repo
        .insert(agent.id, 1, &serde_json::json!({}))
        .await
        .unwrap_err();
    assert!(matches!(err, RepoError::UniqueViolation));
}

#[tokio::test]
async fn insert_nonexistent_agent_fk_violation() {
    let fx = setup_pg().await;
    let err = PgAgentSnapshotRepo { pool: fx.pool }
        .insert(Uuid::new_v4(), 1, &serde_json::json!({}))
        .await
        .unwrap_err();
    assert!(matches!(err, RepoError::ForeignKeyViolation));
}

#[tokio::test]
async fn cascade_delete_when_agent_removed() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13700000004").await;
    let agent = create_full_agent(&fx.pool, owner, "cascade").await;
    let repo = PgAgentSnapshotRepo {
        pool: fx.pool.clone(),
    };
    let snap = repo
        .insert(agent.id, 1, &serde_json::json!({}))
        .await
        .unwrap();
    sqlx::query("DELETE FROM agents WHERE id = $1")
        .bind(agent.id)
        .execute(&fx.pool)
        .await
        .unwrap();
    assert!(repo.find_by_id(snap.id).await.unwrap().is_none());
}

#[tokio::test]
async fn delete_removes_snapshot() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13700000005").await;
    let agent = create_full_agent(&fx.pool, owner, "del").await;
    let repo = PgAgentSnapshotRepo {
        pool: fx.pool.clone(),
    };
    let snap = repo
        .insert(agent.id, 1, &serde_json::json!({}))
        .await
        .unwrap();
    repo.delete(snap.id).await.unwrap();
    assert!(repo.find_by_id(snap.id).await.unwrap().is_none());
}

#[tokio::test]
async fn delete_missing_returns_not_found() {
    let fx = setup_pg().await;
    let err = PgAgentSnapshotRepo { pool: fx.pool }
        .delete(Uuid::new_v4())
        .await
        .unwrap_err();
    assert!(matches!(err, RepoError::NotFound));
}
