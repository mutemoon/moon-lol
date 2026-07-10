//! Agent 子系统 repository 集成测试（testcontainers 真 PG）。

mod common;

use common::setup_pg;
use lol_web_server::domain::RepoError;
use lol_web_server::domain::agent::{AgentInput, AgentType};
use lol_web_server::domain::spawn_preset::Visibility;
use lol_web_server::repository::agent_repo::{AgentRepo, PgAgentRepo};
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

/// 创建一个完整的 Agent，返回 agent。
async fn create_full_agent(
    pool: &sqlx::PgPool,
    owner: i32,
    name: &str,
    champion: &str,
) -> lol_web_server::domain::agent::Agent {
    let agent_repo = PgAgentRepo { pool: pool.clone() };

    agent_repo
        .insert(
            owner,
            &AgentInput {
                name: name.into(),
                champion: champion.into(),
                agent_type: AgentType::Llm,
                prompt: "test_prompt".into(),
                model: "test_model".into(),
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
    let repo = PgAgentRepo { pool: fx.pool };
    assert!(repo.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[tokio::test]
async fn insert_find_roundtrip() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600000001").await;
    let agent = create_full_agent(&fx.pool, owner, "锐雯", "Riven").await;

    let found = PgAgentRepo {
        pool: fx.pool.clone(),
    }
    .find_by_id(agent.id)
    .await
    .unwrap()
    .unwrap();
    assert_eq!(found.name, "锐雯");
    assert_eq!(found.champion, "Riven");
    assert_eq!(found.owner_id, owner);
    assert_eq!(found.agent_type, AgentType::Llm);
    assert_eq!(found.prompt, "test_prompt");
}

#[tokio::test]
async fn list_by_owner() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600000002").await;
    let other = create_user(&fx.pool, "13600000003").await;
    create_full_agent(&fx.pool, owner, "A", "Riven").await;
    create_full_agent(&fx.pool, owner, "B", "Fiora").await;
    create_full_agent(&fx.pool, other, "C", "Riven").await;

    let repo = PgAgentRepo {
        pool: fx.pool.clone(),
    };
    assert_eq!(repo.list_by_owner(owner).await.unwrap().len(), 2);
    assert_eq!(repo.list_by_owner(other).await.unwrap().len(), 1);
}

#[tokio::test]
async fn count_by_owner() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600000004").await;
    let repo = PgAgentRepo {
        pool: fx.pool.clone(),
    };
    assert_eq!(repo.count_by_owner(owner).await.unwrap(), 0);
    create_full_agent(&fx.pool, owner, "A", "Riven").await;
    assert_eq!(repo.count_by_owner(owner).await.unwrap(), 1);
}

#[tokio::test]
async fn duplicate_name_per_owner_rejected() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600000005").await;
    create_full_agent(&fx.pool, owner, "dup", "Riven").await;

    let err = PgAgentRepo {
        pool: fx.pool.clone(),
    }
    .insert(
        owner,
        &AgentInput {
            name: "dup".into(),
            champion: "Riven".into(),
            agent_type: AgentType::Llm,
            prompt: "".into(),
            model: "".into(),
            config_json: serde_json::json!({}),
            visibility: Visibility::Private,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, RepoError::UniqueViolation));
}

#[tokio::test]
async fn update_changes_fields() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600000007").await;
    let agent = create_full_agent(&fx.pool, owner, "orig", "Riven").await;

    PgAgentRepo {
        pool: fx.pool.clone(),
    }
    .update(
        agent.id,
        &AgentInput {
            name: "updated".into(),
            champion: "Fiora".into(),
            agent_type: AgentType::Script,
            prompt: "new_prompt".into(),
            model: "new_model".into(),
            config_json: serde_json::json!({"x": 1}),
            visibility: Visibility::Public,
        },
    )
    .await
    .unwrap();

    let found = PgAgentRepo {
        pool: fx.pool.clone(),
    }
    .find_by_id(agent.id)
    .await
    .unwrap()
    .unwrap();
    assert_eq!(found.name, "updated");
    assert_eq!(found.champion, "Fiora");
    assert_eq!(found.agent_type, AgentType::Script);
    assert_eq!(found.prompt, "new_prompt");
    assert_eq!(found.config_json["x"], 1);
    assert_eq!(found.visibility, Visibility::Public);
}

#[tokio::test]
async fn update_visibility() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600000008").await;
    let agent = create_full_agent(&fx.pool, owner, "v", "Riven").await;

    PgAgentRepo {
        pool: fx.pool.clone(),
    }
    .update_visibility(agent.id, Visibility::Public)
    .await
    .unwrap();

    let found = PgAgentRepo {
        pool: fx.pool.clone(),
    }
    .find_by_id(agent.id)
    .await
    .unwrap()
    .unwrap();
    assert_eq!(found.visibility, Visibility::Public);
}

#[tokio::test]
async fn delete_removes() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600000009").await;
    let agent = create_full_agent(&fx.pool, owner, "del", "Riven").await;
    PgAgentRepo {
        pool: fx.pool.clone(),
    }
    .delete(agent.id)
    .await
    .unwrap();
    assert!(
        PgAgentRepo {
            pool: fx.pool.clone()
        }
        .find_by_id(agent.id)
        .await
        .unwrap()
        .is_none()
    );
}

#[tokio::test]
async fn cascade_delete_when_user_removed() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600000010").await;
    let agent = create_full_agent(&fx.pool, owner, "orphan", "Riven").await;
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(owner)
        .execute(&fx.pool)
        .await
        .unwrap();
    assert!(
        PgAgentRepo { pool: fx.pool }
            .find_by_id(agent.id)
            .await
            .unwrap()
            .is_none()
    );
}
