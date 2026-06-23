//! Agent 子系统 repository 集成测试（testcontainers 真 PG）。

mod common;

use common::setup_pg;
use lol_web_server::domain::RepoError;
use lol_web_server::domain::agent::AgentInput;
use lol_web_server::domain::agent_config::{AgentConfigInput, AgentType};
use lol_web_server::domain::spawn_preset::{SpawnPresetInput, Team, Visibility};
use lol_web_server::repository::agent_config_repo::{AgentConfigRepo, PgAgentConfigRepo};
use lol_web_server::repository::agent_repo::{AgentRepo, PgAgentRepo};
use lol_web_server::repository::spawn_preset_repo::{PgSpawnPresetRepo, SpawnPresetRepo};
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

/// 创建一个完整的 Agent（含依赖的 config + spawn），返回 agent。
/// 用 name 派生唯一的 config/spawn 名，避免同 owner 重复。
async fn create_full_agent(
    pool: &sqlx::PgPool,
    owner: i32,
    name: &str,
    champion: &str,
) -> lol_web_server::domain::agent::Agent {
    let config_repo = PgAgentConfigRepo { pool: pool.clone() };
    let spawn_repo = PgSpawnPresetRepo { pool: pool.clone() };
    let agent_repo = PgAgentRepo { pool: pool.clone() };

    let config = config_repo
        .insert(
            owner,
            &AgentConfigInput {
                name: format!("cfg_{name}"),
                agent_type: AgentType::Llm,
                prompt: "".into(),
                preamble: "".into(),
                model: "".into(),
                config_json: serde_json::json!({}),
                visibility: Visibility::Private,
            },
        )
        .await
        .unwrap();

    let spawn = spawn_repo
        .insert(
            owner,
            &SpawnPresetInput {
                name: format!("sp_{name}"),
                x: 1000.0,
                z: 1000.0,
                team: Team::Order,
                visibility: Visibility::Private,
            },
        )
        .await
        .unwrap();

    agent_repo
        .insert(
            owner,
            &AgentInput {
                name: name.into(),
                champion: champion.into(),
                agent_config_id: config.id,
                spawn_preset_id: Some(spawn.id),
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

    // 再插一个同名 agent（需新建 config/spawn 因为 owner+name 唯一）
    let config_repo = PgAgentConfigRepo {
        pool: fx.pool.clone(),
    };
    let cfg2 = config_repo
        .insert(
            owner,
            &AgentConfigInput {
                name: "cfg2".into(),
                agent_type: AgentType::Llm,
                prompt: "".into(),
                preamble: "".into(),
                model: "".into(),
                config_json: serde_json::json!({}),
                visibility: Visibility::Private,
            },
        )
        .await
        .unwrap();

    let err = PgAgentRepo {
        pool: fx.pool.clone(),
    }
    .insert(
        owner,
        &AgentInput {
            name: "dup".into(),
            champion: "Riven".into(),
            agent_config_id: cfg2.id,
            spawn_preset_id: None,
            visibility: Visibility::Private,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, RepoError::UniqueViolation));
}

#[tokio::test]
async fn insert_with_nonexistent_config_fk_violation() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13600000006").await;
    let err = PgAgentRepo {
        pool: fx.pool.clone(),
    }
    .insert(
        owner,
        &AgentInput {
            name: "orphan".into(),
            champion: "Riven".into(),
            agent_config_id: Uuid::new_v4(), // 不存在的 config
            spawn_preset_id: None,
            visibility: Visibility::Private,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, RepoError::ForeignKeyViolation));
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
            agent_config_id: agent.agent_config_id,
            spawn_preset_id: agent.spawn_preset_id,
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
