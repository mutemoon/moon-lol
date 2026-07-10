//! Match 子系统 repository 集成测试（testcontainers 真 PG）。

mod common;

use common::setup_pg;
use lol_web_server::domain::RepoError;
use lol_web_server::domain::agent::{AgentInput, AgentType};
use lol_web_server::domain::match_::{MatchForm, MatchStatus, ParticipantResult, Winner};
use lol_web_server::domain::spawn_preset::{Team, Visibility};
use lol_web_server::repository::agent_repo::{AgentRepo, PgAgentRepo};
use lol_web_server::repository::agent_snapshot_repo::{AgentSnapshotRepo, PgAgentSnapshotRepo};
use lol_web_server::repository::match_repo::{
    MatchEventInput, MatchEventRepo, MatchInput, MatchParticipantRepo, MatchRepo, ParticipantInput,
    PgMatchEventRepo, PgMatchParticipantRepo, PgMatchRepo,
};
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

async fn create_agent_with_snapshot(pool: &sqlx::PgPool, owner: i32, name: &str) -> (Uuid, Uuid) {
    let agent = PgAgentRepo { pool: pool.clone() }
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
        .unwrap();
    let snap = PgAgentSnapshotRepo { pool: pool.clone() }
        .insert(agent.id, 1, &serde_json::json!({}))
        .await
        .unwrap();
    (agent.id, snap.id)
}

fn sample_input() -> MatchInput {
    MatchInput {
        form: MatchForm::Local,
        room_id: None,
        mode: "1v1".into(),
        scenario_id: None,
        win_condition: None,
    }
}

#[tokio::test]
async fn find_missing_returns_none() {
    let fx = setup_pg().await;
    assert!(
        PgMatchRepo { pool: fx.pool }
            .find_by_id(Uuid::new_v4())
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn insert_find_roundtrip() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13800000001").await;
    let repo = PgMatchRepo {
        pool: fx.pool.clone(),
    };
    let m = repo.insert(owner, &sample_input()).await.unwrap();
    assert_eq!(m.status, MatchStatus::Pending);
    assert_eq!(m.form, MatchForm::Local);
    assert_eq!(repo.find_by_id(m.id).await.unwrap().unwrap().mode, "1v1");
}

#[tokio::test]
async fn status_pending_to_running_to_finished() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13800000002").await;
    let repo = PgMatchRepo {
        pool: fx.pool.clone(),
    };
    let m = repo.insert(owner, &sample_input()).await.unwrap();
    repo.update_status(m.id, MatchStatus::Pending, MatchStatus::Running)
        .await
        .unwrap();
    assert_eq!(
        repo.find_by_id(m.id).await.unwrap().unwrap().status,
        MatchStatus::Running
    );
    repo.update_result(m.id, Winner::Order).await.unwrap();
    let after = repo.find_by_id(m.id).await.unwrap().unwrap();
    assert_eq!(after.status, MatchStatus::Finished);
    assert_eq!(after.winner_team, Some(Winner::Order));
}

#[tokio::test]
async fn status_update_wrong_from_returns_not_found() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13800000003").await;
    let repo = PgMatchRepo {
        pool: fx.pool.clone(),
    };
    let m = repo.insert(owner, &sample_input()).await.unwrap();
    let err = repo
        .update_status(m.id, MatchStatus::Running, MatchStatus::Paused)
        .await
        .unwrap_err();
    assert!(matches!(err, RepoError::NotFound));
}

#[tokio::test]
async fn abort_sets_reason() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13800000004").await;
    let repo = PgMatchRepo {
        pool: fx.pool.clone(),
    };
    let m = repo.insert(owner, &sample_input()).await.unwrap();
    repo.update_status(m.id, MatchStatus::Pending, MatchStatus::Running)
        .await
        .unwrap();
    repo.update_abort(m.id, MatchStatus::Running, "crash")
        .await
        .unwrap();
    let after = repo.find_by_id(m.id).await.unwrap().unwrap();
    assert_eq!(after.status, MatchStatus::Aborted);
    assert_eq!(after.abort_reason.as_deref(), Some("crash"));
}

#[tokio::test]
async fn participant_result_by_team() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13800000005").await;
    let (agent_a, snap_a) = create_agent_with_snapshot(&fx.pool, owner, "A").await;
    let (agent_b, snap_b) = create_agent_with_snapshot(&fx.pool, owner, "B").await;
    let mrepo = PgMatchRepo {
        pool: fx.pool.clone(),
    };
    let prepo = PgMatchParticipantRepo {
        pool: fx.pool.clone(),
    };
    let m = mrepo.insert(owner, &sample_input()).await.unwrap();
    prepo
        .insert(
            m.id,
            &ParticipantInput {
                agent_snapshot_id: snap_a,
                agent_id: agent_a,
                user_id: owner,
                team: Team::Order,
            },
        )
        .await
        .unwrap();
    prepo
        .insert(
            m.id,
            &ParticipantInput {
                agent_snapshot_id: snap_b,
                agent_id: agent_b,
                user_id: owner,
                team: Team::Chaos,
            },
        )
        .await
        .unwrap();
    prepo
        .update_result_by_team(m.id, Team::Order, ParticipantResult::Win)
        .await
        .unwrap();
    prepo
        .update_result_by_team(m.id, Team::Chaos, ParticipantResult::Loss)
        .await
        .unwrap();
    let parts = prepo.find_by_match(m.id).await.unwrap();
    assert_eq!(
        parts.iter().find(|p| p.team == Team::Order).unwrap().result,
        Some(ParticipantResult::Win)
    );
    assert_eq!(
        parts.iter().find(|p| p.team == Team::Chaos).unwrap().result,
        Some(ParticipantResult::Loss)
    );
}

#[tokio::test]
async fn participant_cascade_delete_with_match() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13800000006").await;
    let (agent, snap) = create_agent_with_snapshot(&fx.pool, owner, "C").await;
    let mrepo = PgMatchRepo {
        pool: fx.pool.clone(),
    };
    let prepo = PgMatchParticipantRepo {
        pool: fx.pool.clone(),
    };
    let m = mrepo.insert(owner, &sample_input()).await.unwrap();
    prepo
        .insert(
            m.id,
            &ParticipantInput {
                agent_snapshot_id: snap,
                agent_id: agent,
                user_id: owner,
                team: Team::Order,
            },
        )
        .await
        .unwrap();
    sqlx::query("DELETE FROM matches WHERE id = $1")
        .bind(m.id)
        .execute(&fx.pool)
        .await
        .unwrap();
    assert!(prepo.find_by_match(m.id).await.unwrap().is_empty());
}

#[tokio::test]
async fn event_seq_increments() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13800000007").await;
    let mrepo = PgMatchRepo {
        pool: fx.pool.clone(),
    };
    let erepo = PgMatchEventRepo {
        pool: fx.pool.clone(),
    };
    let m = mrepo.insert(owner, &sample_input()).await.unwrap();
    let e1 = erepo
        .append(
            m.id,
            &MatchEventInput {
                event_type: "move".into(),
                agent_id: None,
                payload: serde_json::json!({}),
                game_time_ms: 100,
            },
        )
        .await
        .unwrap();
    let e2 = erepo
        .append(
            m.id,
            &MatchEventInput {
                event_type: "attack".into(),
                agent_id: None,
                payload: serde_json::json!({}),
                game_time_ms: 200,
            },
        )
        .await
        .unwrap();
    assert_eq!(e1.seq, 1);
    assert_eq!(e2.seq, 2);
    let all = erepo.list_by_match(m.id, 0, 100).await.unwrap();
    assert_eq!(all.len(), 2);
    let tail = erepo.list_by_match(m.id, 2, 100).await.unwrap();
    assert_eq!(tail.len(), 1);
}

#[tokio::test]
async fn event_cascade_delete_with_match() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13800000008").await;
    let mrepo = PgMatchRepo {
        pool: fx.pool.clone(),
    };
    let erepo = PgMatchEventRepo {
        pool: fx.pool.clone(),
    };
    let m = mrepo.insert(owner, &sample_input()).await.unwrap();
    erepo
        .append(
            m.id,
            &MatchEventInput {
                event_type: "x".into(),
                agent_id: None,
                payload: serde_json::json!({}),
                game_time_ms: 0,
            },
        )
        .await
        .unwrap();
    sqlx::query("DELETE FROM matches WHERE id = $1")
        .bind(m.id)
        .execute(&fx.pool)
        .await
        .unwrap();
    assert!(erepo.list_by_match(m.id, 0, 100).await.unwrap().is_empty());
}

#[tokio::test]
async fn match_cascade_delete_when_user_removed() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13800000009").await;
    let mrepo = PgMatchRepo {
        pool: fx.pool.clone(),
    };
    let m = mrepo.insert(owner, &sample_input()).await.unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(owner)
        .execute(&fx.pool)
        .await
        .unwrap();
    assert!(mrepo.find_by_id(m.id).await.unwrap().is_none());
}
