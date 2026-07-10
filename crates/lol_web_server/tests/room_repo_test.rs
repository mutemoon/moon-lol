//! Room 子系统 repository 集成测试（testcontainers 真 PG）。

mod common;

use common::setup_pg;
use lol_web_server::domain::room::{RoomConstraints, RoomStatus, TeamPolicy};
use lol_web_server::domain::spawn_preset::Team;
use lol_web_server::repository::room_repo::{CreateRoomInput, PgRoomRepo, RoomRepo};
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

fn sample_input() -> CreateRoomInput {
    CreateRoomInput {
        name: "测试房间".into(),
        constraints: RoomConstraints::default(),
    }
}

#[tokio::test]
async fn find_missing_returns_none() {
    let fx = setup_pg().await;
    let repo = PgRoomRepo { pool: fx.pool };
    assert!(repo.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[tokio::test]
async fn insert_includes_owner_as_member() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13500000001").await;
    let repo = PgRoomRepo {
        pool: fx.pool.clone(),
    };
    let room = repo.insert(owner, &sample_input()).await.unwrap();
    assert_eq!(room.owner_id, owner);
    assert_eq!(room.status, RoomStatus::Lobby);
    assert_eq!(room.invite_code.len(), 6);
    // 房主自动加入
    assert_eq!(repo.count_members(room.id).await.unwrap(), 1);
    assert!(repo.is_member(room.id, owner).await.unwrap());
}

#[tokio::test]
async fn find_by_invite_code() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13500000002").await;
    let repo = PgRoomRepo {
        pool: fx.pool.clone(),
    };
    let room = repo.insert(owner, &sample_input()).await.unwrap();
    let found = repo
        .find_by_invite_code(&room.invite_code)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.id, room.id);
}

#[tokio::test]
async fn join_and_leave_member() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13500000003").await;
    let member = create_user(&fx.pool, "13500000004").await;
    let repo = PgRoomRepo {
        pool: fx.pool.clone(),
    };
    let room = repo.insert(owner, &sample_input()).await.unwrap();

    repo.add_member(room.id, member).await.unwrap();
    assert_eq!(repo.count_members(room.id).await.unwrap(), 2);
    assert!(repo.is_member(room.id, member).await.unwrap());

    repo.remove_member(room.id, member).await.unwrap();
    assert_eq!(repo.count_members(room.id).await.unwrap(), 1);
    assert!(!repo.is_member(room.id, member).await.unwrap());
}

#[tokio::test]
async fn add_and_remove_slot() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13500000005").await;
    let repo = PgRoomRepo {
        pool: fx.pool.clone(),
    };
    let room = repo.insert(owner, &sample_input()).await.unwrap();

    // 需要先有 agent 才能 add_slot（FK 约束）
    let agent_id = create_agent_for_user(&fx.pool, owner).await;

    let slot = repo
        .add_slot(room.id, owner, agent_id, Team::Order)
        .await
        .unwrap();
    assert_eq!(slot.team, Team::Order);
    assert_eq!(repo.count_slots_by_member(room.id, owner).await.unwrap(), 1);

    repo.remove_slot(slot.id).await.unwrap();
    assert_eq!(repo.count_slots_by_member(room.id, owner).await.unwrap(), 0);
}

#[tokio::test]
async fn member_existing_team_detected() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13500000006").await;
    let repo = PgRoomRepo {
        pool: fx.pool.clone(),
    };
    let room = repo.insert(owner, &sample_input()).await.unwrap();
    let agent_id = create_agent_for_user(&fx.pool, owner).await;

    repo.add_slot(room.id, owner, agent_id, Team::Chaos)
        .await
        .unwrap();
    let team = repo.member_existing_team(room.id, owner).await.unwrap();
    assert_eq!(team, Some(Team::Chaos));
}

#[tokio::test]
async fn update_status() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13500000007").await;
    let repo = PgRoomRepo {
        pool: fx.pool.clone(),
    };
    let room = repo.insert(owner, &sample_input()).await.unwrap();
    repo.update_status(room.id, RoomStatus::Running)
        .await
        .unwrap();
    let found = repo.find_by_id(room.id).await.unwrap().unwrap();
    assert_eq!(found.status, RoomStatus::Running);
}

#[tokio::test]
async fn update_constraints() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13500000008").await;
    let repo = PgRoomRepo {
        pool: fx.pool.clone(),
    };
    let room = repo.insert(owner, &sample_input()).await.unwrap();

    let mut c = RoomConstraints::default();
    c.max_members = 5;
    c.team_policy = TeamPolicy::SingleTeam;
    repo.update_constraints(room.id, c).await.unwrap();

    let found = repo.find_by_id(room.id).await.unwrap().unwrap();
    assert_eq!(found.constraints.max_members, 5);
    assert_eq!(found.constraints.team_policy, TeamPolicy::SingleTeam);
}

#[tokio::test]
async fn delete_cascades_members_and_slots() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13500000009").await;
    let repo = PgRoomRepo {
        pool: fx.pool.clone(),
    };
    let room = repo.insert(owner, &sample_input()).await.unwrap();
    let agent_id = create_agent_for_user(&fx.pool, owner).await;
    repo.add_slot(room.id, owner, agent_id, Team::Order)
        .await
        .unwrap();

    repo.delete(room.id).await.unwrap();
    assert!(repo.find_by_id(room.id).await.unwrap().is_none());
    // 成员和槽位应级联删除
    assert_eq!(repo.count_members(room.id).await.unwrap(), 0);
    assert!(repo.list_slots(room.id).await.unwrap().is_empty());
}

#[tokio::test]
async fn list_lobby_shows_visible_rooms() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13500000010").await;
    let repo = PgRoomRepo {
        pool: fx.pool.clone(),
    };
    // 可见房间
    repo.insert(owner, &sample_input()).await.unwrap();
    // 不可见房间
    let mut input = sample_input();
    input.constraints.lobby_visible = false;
    repo.insert(owner, &input).await.unwrap();

    let lobby = repo.list_lobby().await.unwrap();
    assert_eq!(lobby.len(), 1);
}

/// 为用户创建一个完整 agent，返回 agent_id。
async fn create_agent_for_user(pool: &sqlx::PgPool, owner: i32) -> Uuid {
    use lol_web_server::domain::agent::{AgentInput, AgentType};
    use lol_web_server::domain::spawn_preset::Visibility;
    use lol_web_server::repository::agent_repo::{AgentRepo, PgAgentRepo};

    let agent = PgAgentRepo { pool: pool.clone() }
        .insert(
            owner,
            &AgentInput {
                name: "room_agent".into(),
                champion: "Riven".into(),
                agent_type: AgentType::Llm,
                prompt: "prompt".into(),
                model: "model".into(),
                config_json: serde_json::json!({}),
                visibility: Visibility::Private,
            },
        )
        .await
        .unwrap();
    agent.id
}
