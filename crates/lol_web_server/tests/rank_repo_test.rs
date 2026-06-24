//! Rank 子系统 repository 集成测试。

mod common;

use chrono::{Duration, Utc};
use common::setup_pg;
use lol_web_server::domain::rank::{QueueStatus, SeasonStatus};
use lol_web_server::repository::rank_repo::{
    EloRepo, NewQueueEntry, NewSeason, PgEloRepo, PgRankQueueRepo, PgSeasonRepo, RankQueueRepo,
    SeasonRepo,
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

/// 创建 agent，返回 agent_id。
async fn create_agent(pool: &sqlx::PgPool, owner: i32, name: &str) -> Uuid {
    use lol_web_server::domain::agent::{AgentInput, AgentType};
    use lol_web_server::domain::spawn_preset::Visibility;
    use lol_web_server::repository::agent_repo::{AgentRepo, PgAgentRepo};

    PgAgentRepo { pool: pool.clone() }
        .insert(
            owner,
            &AgentInput {
                name: name.into(),
                champion: "Riven".into(),
                agent_type: AgentType::Llm,
                prompt: "prompt".into(),
                preamble: "preamble".into(),
                model: "model".into(),
                config_json: serde_json::json!({}),
                visibility: Visibility::Private,
            },
        )
        .await
        .unwrap()
        .id
}

/// 创建 agent snapshot，返回 snapshot_id。
async fn create_snapshot(pool: &sqlx::PgPool, agent_id: Uuid) -> Uuid {
    use lol_web_server::repository::agent_snapshot_repo::{AgentSnapshotRepo, PgAgentSnapshotRepo};
    PgAgentSnapshotRepo { pool: pool.clone() }
        .insert(agent_id, 1, &serde_json::json!({"champion": "Riven"}))
        .await
        .unwrap()
        .id
}

fn sample_season() -> NewSeason {
    NewSeason {
        name: "2026 夏季赛".into(),
        mode: "top_solo".into(),
        starts_at: Utc::now() - Duration::days(1),
        ends_at: Utc::now() + Duration::days(90),
    }
}

// ── Season ──

#[tokio::test]
async fn season_insert_find_roundtrip() {
    let fx = setup_pg().await;
    let repo = PgSeasonRepo {
        pool: fx.pool.clone(),
    };
    let s = repo.insert(&sample_season()).await.unwrap();
    assert_eq!(s.status, SeasonStatus::Scheduled);
    let found = repo.find_by_id(s.id).await.unwrap().unwrap();
    assert_eq!(found.name, "2026 夏季赛");
}

#[tokio::test]
async fn season_find_current_active() {
    let fx = setup_pg().await;
    let repo = PgSeasonRepo {
        pool: fx.pool.clone(),
    };
    let s = repo.insert(&sample_season()).await.unwrap();
    repo.update_status(s.id, SeasonStatus::Active)
        .await
        .unwrap();
    let current = repo.find_current("top_solo").await.unwrap().unwrap();
    assert_eq!(current.id, s.id);
}

#[tokio::test]
async fn season_find_current_none_when_not_active() {
    let fx = setup_pg().await;
    let repo = PgSeasonRepo {
        pool: fx.pool.clone(),
    };
    repo.insert(&sample_season()).await.unwrap(); // scheduled，未激活
    assert!(repo.find_current("top_solo").await.unwrap().is_none());
}

// ── RankQueue ──

#[tokio::test]
async fn queue_enqueue_find_dequeue() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13400000001").await;
    let agent_id = create_agent(&fx.pool, owner, "A").await;
    let snap_id = create_snapshot(&fx.pool, agent_id).await;

    let season_repo = PgSeasonRepo {
        pool: fx.pool.clone(),
    };
    let s = season_repo.insert(&sample_season()).await.unwrap();

    let q_repo = PgRankQueueRepo {
        pool: fx.pool.clone(),
    };
    let entry = q_repo
        .enqueue(&NewQueueEntry {
            agent_id,
            agent_snapshot_id: snap_id,
            user_id: owner,
            mode: "top_solo".into(),
            season_id: s.id,
        })
        .await
        .unwrap();
    assert_eq!(entry.status, QueueStatus::Queued);

    let found = q_repo.find_by_agent(agent_id).await.unwrap().unwrap();
    assert_eq!(found.id, entry.id);

    q_repo.dequeue(agent_id).await.unwrap();
    assert!(q_repo.find_by_agent(agent_id).await.unwrap().is_none());
}

#[tokio::test]
async fn queue_duplicate_snapshot_season_rejected() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13400000002").await;
    let agent_id = create_agent(&fx.pool, owner, "dup").await;
    let snap_id = create_snapshot(&fx.pool, agent_id).await;

    let season_repo = PgSeasonRepo {
        pool: fx.pool.clone(),
    };
    let s = season_repo.insert(&sample_season()).await.unwrap();

    let q_repo = PgRankQueueRepo {
        pool: fx.pool.clone(),
    };
    q_repo
        .enqueue(&NewQueueEntry {
            agent_id,
            agent_snapshot_id: snap_id,
            user_id: owner,
            mode: "top_solo".into(),
            season_id: s.id,
        })
        .await
        .unwrap();
    let err = q_repo
        .enqueue(&NewQueueEntry {
            agent_id,
            agent_snapshot_id: snap_id,
            user_id: owner,
            mode: "top_solo".into(),
            season_id: s.id,
        })
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        lol_web_server::domain::RepoError::UniqueViolation
    ));
}

#[tokio::test]
async fn queue_list_queued_by_mode() {
    let fx = setup_pg().await;
    let owner_a = create_user(&fx.pool, "13400000003").await;
    let owner_b = create_user(&fx.pool, "13400000004").await;
    let agent_a = create_agent(&fx.pool, owner_a, "QA").await;
    let agent_b = create_agent(&fx.pool, owner_b, "QB").await;
    let snap_a = create_snapshot(&fx.pool, agent_a).await;
    let snap_b = create_snapshot(&fx.pool, agent_b).await;

    let s = PgSeasonRepo {
        pool: fx.pool.clone(),
    }
    .insert(&sample_season())
    .await
    .unwrap();
    let q_repo = PgRankQueueRepo {
        pool: fx.pool.clone(),
    };
    q_repo
        .enqueue(&NewQueueEntry {
            agent_id: agent_a,
            agent_snapshot_id: snap_a,
            user_id: owner_a,
            mode: "top_solo".into(),
            season_id: s.id,
        })
        .await
        .unwrap();
    q_repo
        .enqueue(&NewQueueEntry {
            agent_id: agent_b,
            agent_snapshot_id: snap_b,
            user_id: owner_b,
            mode: "top_solo".into(),
            season_id: s.id,
        })
        .await
        .unwrap();

    let queued = q_repo.list_queued("top_solo").await.unwrap();
    assert_eq!(queued.len(), 2);
}

#[tokio::test]
async fn queue_find_opponent_excludes_self() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13400000005").await;
    let other = create_user(&fx.pool, "13400000006").await;
    let agent_a = create_agent(&fx.pool, owner, "OA").await;
    let agent_b = create_agent(&fx.pool, other, "OB").await;
    let snap_a = create_snapshot(&fx.pool, agent_a).await;
    let snap_b = create_snapshot(&fx.pool, agent_b).await;

    let s = PgSeasonRepo {
        pool: fx.pool.clone(),
    }
    .insert(&sample_season())
    .await
    .unwrap();
    let q_repo = PgRankQueueRepo {
        pool: fx.pool.clone(),
    };
    q_repo
        .enqueue(&NewQueueEntry {
            agent_id: agent_a,
            agent_snapshot_id: snap_a,
            user_id: owner,
            mode: "top_solo".into(),
            season_id: s.id,
        })
        .await
        .unwrap();
    q_repo
        .enqueue(&NewQueueEntry {
            agent_id: agent_b,
            agent_snapshot_id: snap_b,
            user_id: other,
            mode: "top_solo".into(),
            season_id: s.id,
        })
        .await
        .unwrap();

    let opp = q_repo
        .find_opponent("top_solo", agent_a, 1200.0, 100.0)
        .await
        .unwrap();
    assert!(opp.is_some());
    assert_ne!(opp.unwrap().agent_id, agent_a);
}

#[tokio::test]
async fn queue_find_opponent_none_when_empty() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13400000007").await;
    let agent_a = create_agent(&fx.pool, owner, "OE").await;
    let q_repo = PgRankQueueRepo { pool: fx.pool };
    assert!(
        q_repo
            .find_opponent("top_solo", agent_a, 1200.0, 100.0)
            .await
            .unwrap()
            .is_none()
    );
}

// ── ELO ──

#[tokio::test]
async fn elo_upsert_initial_creates_default() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13400000008").await;
    let agent_id = create_agent(&fx.pool, owner, "E1").await;
    let s = PgSeasonRepo {
        pool: fx.pool.clone(),
    }
    .insert(&sample_season())
    .await
    .unwrap();

    let elo_repo = PgEloRepo {
        pool: fx.pool.clone(),
    };
    let rating = elo_repo
        .upsert_initial(agent_id, "top_solo", s.id)
        .await
        .unwrap();
    assert_eq!(rating.rating, 1200.0);
    assert_eq!(rating.wins, 0);
}

#[tokio::test]
async fn elo_upsert_initial_idempotent() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13400000009").await;
    let agent_id = create_agent(&fx.pool, owner, "E2").await;
    let s = PgSeasonRepo {
        pool: fx.pool.clone(),
    }
    .insert(&sample_season())
    .await
    .unwrap();

    let elo_repo = PgEloRepo {
        pool: fx.pool.clone(),
    };
    let r1 = elo_repo
        .upsert_initial(agent_id, "top_solo", s.id)
        .await
        .unwrap();
    let r2 = elo_repo
        .upsert_initial(agent_id, "top_solo", s.id)
        .await
        .unwrap();
    assert_eq!(r1.id, r2.id, "重复 upsert 应返回同一行");
}

#[tokio::test]
async fn elo_update_after_match() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13400000010").await;
    let agent_id = create_agent(&fx.pool, owner, "E3").await;
    let s = PgSeasonRepo {
        pool: fx.pool.clone(),
    }
    .insert(&sample_season())
    .await
    .unwrap();

    let elo_repo = PgEloRepo {
        pool: fx.pool.clone(),
    };
    let r = elo_repo
        .upsert_initial(agent_id, "top_solo", s.id)
        .await
        .unwrap();
    elo_repo
        .update_after_match(r.id, 1232.0, true, false)
        .await
        .unwrap();

    let found = elo_repo
        .find(agent_id, "top_solo", s.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.rating, 1232.0);
    assert_eq!(found.wins, 1);
}

#[tokio::test]
async fn elo_leaderboard_sorted_by_rating() {
    let fx = setup_pg().await;
    let owner_a = create_user(&fx.pool, "13400000011").await;
    let owner_b = create_user(&fx.pool, "13400000012").await;
    let agent_a = create_agent(&fx.pool, owner_a, "LA").await;
    let agent_b = create_agent(&fx.pool, owner_b, "LB").await;
    let s = PgSeasonRepo {
        pool: fx.pool.clone(),
    }
    .insert(&sample_season())
    .await
    .unwrap();

    let elo_repo = PgEloRepo {
        pool: fx.pool.clone(),
    };
    let ra = elo_repo
        .upsert_initial(agent_a, "top_solo", s.id)
        .await
        .unwrap();
    let rb = elo_repo
        .upsert_initial(agent_b, "top_solo", s.id)
        .await
        .unwrap();
    elo_repo
        .update_after_match(ra.id, 1800.0, true, false)
        .await
        .unwrap();
    elo_repo
        .update_after_match(rb.id, 1500.0, true, false)
        .await
        .unwrap();

    let board = elo_repo.leaderboard("top_solo", s.id, 10).await.unwrap();
    assert_eq!(board.len(), 2);
    assert_eq!(board[0].agent_id, agent_a, "高分在前");
    assert_eq!(board[1].agent_id, agent_b);
}
