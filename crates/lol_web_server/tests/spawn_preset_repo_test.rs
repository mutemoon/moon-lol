//! SpawnPreset 子系统 repository 集成测试（testcontainers 真 PG）。

mod common;

use common::setup_pg;
use lol_web_server::domain::spawn_preset::{SpawnPresetInput, Team, Visibility};
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

fn sample_input(name: &str) -> SpawnPresetInput {
    SpawnPresetInput {
        name: name.into(),
        x: 1500.0,
        z: 13000.0,
        team: Team::Order,
        visibility: Visibility::Private,
    }
}

#[tokio::test]
async fn find_by_id_missing_returns_none() {
    let fx = setup_pg().await;
    let repo = PgSpawnPresetRepo { pool: fx.pool };
    assert!(repo.find_by_id(Uuid::new_v4()).await.unwrap().is_none());
}

#[tokio::test]
async fn insert_then_find_roundtrip() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13900000001").await;
    let repo = PgSpawnPresetRepo {
        pool: fx.pool.clone(),
    };
    let preset = repo.insert(owner, &sample_input("上路一塔")).await.unwrap();
    assert!(preset.id != Uuid::nil());

    let found = repo.find_by_id(preset.id).await.unwrap().unwrap();
    assert_eq!(found.name, "上路一塔");
    assert_eq!(found.team, Team::Order);
    assert_eq!(found.x, 1500.0);
}

#[tokio::test]
async fn list_by_owner_returns_only_owned() {
    let fx = setup_pg().await;
    let owner_a = create_user(&fx.pool, "13900000002").await;
    let owner_b = create_user(&fx.pool, "13900000003").await;
    let repo = PgSpawnPresetRepo {
        pool: fx.pool.clone(),
    };
    repo.insert(owner_a, &sample_input("A1")).await.unwrap();
    repo.insert(owner_a, &sample_input("A2")).await.unwrap();
    repo.insert(owner_b, &sample_input("B1")).await.unwrap();

    let list_a = repo.list_by_owner(owner_a).await.unwrap();
    assert_eq!(list_a.len(), 2);
    let list_b = repo.list_by_owner(owner_b).await.unwrap();
    assert_eq!(list_b.len(), 1);
}

#[tokio::test]
async fn duplicate_name_per_owner_rejected() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13900000004").await;
    let repo = PgSpawnPresetRepo {
        pool: fx.pool.clone(),
    };
    repo.insert(owner, &sample_input("same")).await.unwrap();
    let err = repo.insert(owner, &sample_input("same")).await.unwrap_err();
    assert!(matches!(
        err,
        lol_web_server::domain::RepoError::UniqueViolation
    ));
}

#[tokio::test]
async fn same_name_different_owners_allowed() {
    let fx = setup_pg().await;
    let owner_a = create_user(&fx.pool, "13900000005").await;
    let owner_b = create_user(&fx.pool, "13900000006").await;
    let repo = PgSpawnPresetRepo {
        pool: fx.pool.clone(),
    };
    repo.insert(owner_a, &sample_input("shared")).await.unwrap();
    repo.insert(owner_b, &sample_input("shared")).await.unwrap();
    // 两个 owner 各自允许同名
}

#[tokio::test]
async fn update_changes_fields() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13900000007").await;
    let repo = PgSpawnPresetRepo {
        pool: fx.pool.clone(),
    };
    let preset = repo.insert(owner, &sample_input("original")).await.unwrap();

    repo.update(
        preset.id,
        &SpawnPresetInput {
            name: "updated".into(),
            x: 2000.0,
            z: 5000.0,
            team: Team::Chaos,
            visibility: Visibility::Public,
        },
    )
    .await
    .unwrap();

    let found = repo.find_by_id(preset.id).await.unwrap().unwrap();
    assert_eq!(found.name, "updated");
    assert_eq!(found.team, Team::Chaos);
    assert_eq!(found.visibility, Visibility::Public);
    assert_eq!(found.x, 2000.0);
}

#[tokio::test]
async fn update_missing_returns_not_found() {
    let fx = setup_pg().await;
    let repo = PgSpawnPresetRepo { pool: fx.pool };
    let err = repo
        .update(Uuid::new_v4(), &sample_input("x"))
        .await
        .unwrap_err();
    assert!(matches!(err, lol_web_server::domain::RepoError::NotFound));
}

#[tokio::test]
async fn delete_removes_preset() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13900000008").await;
    let repo = PgSpawnPresetRepo {
        pool: fx.pool.clone(),
    };
    let preset = repo
        .insert(owner, &sample_input("to-delete"))
        .await
        .unwrap();
    repo.delete(preset.id).await.unwrap();
    assert!(repo.find_by_id(preset.id).await.unwrap().is_none());
}

#[tokio::test]
async fn delete_missing_returns_not_found() {
    let fx = setup_pg().await;
    let repo = PgSpawnPresetRepo { pool: fx.pool };
    let err = repo.delete(Uuid::new_v4()).await.unwrap_err();
    assert!(matches!(err, lol_web_server::domain::RepoError::NotFound));
}

#[tokio::test]
async fn cascade_delete_when_user_removed() {
    let fx = setup_pg().await;
    let owner = create_user(&fx.pool, "13900000009").await;
    let repo = PgSpawnPresetRepo {
        pool: fx.pool.clone(),
    };
    let preset = repo.insert(owner, &sample_input("orphan")).await.unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(owner)
        .execute(&fx.pool)
        .await
        .unwrap();
    assert!(repo.find_by_id(preset.id).await.unwrap().is_none());
}
