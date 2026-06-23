//! User/Auth 子系统 repository 集成测试（testcontainers 真 PG）。

mod common;

use common::setup_pg;
use lol_web_server::domain::auth::User;
use lol_web_server::repository::user_repo::{PgUserRepo, UserRepo};

#[tokio::test]
async fn find_by_phone_returns_none_when_not_exists() {
    let fx = setup_pg().await;
    let repo = PgUserRepo {
        pool: fx.pool.clone(),
    };
    assert!(repo.find_by_phone("13800000000").await.unwrap().is_none());
}

#[tokio::test]
async fn insert_then_find_by_phone_roundtrip() {
    let fx = setup_pg().await;
    let repo = PgUserRepo {
        pool: fx.pool.clone(),
    };
    let user = repo.insert("13800000020", "hash_a").await.unwrap();
    assert!(user.id > 0);

    let (found, hash) = repo.find_by_phone("13800000020").await.unwrap().unwrap();
    assert_eq!(found.id, user.id);
    assert_eq!(found.phone, "13800000020");
    assert_eq!(hash, "hash_a");
}

#[tokio::test]
async fn find_by_id_roundtrip() {
    let fx = setup_pg().await;
    let repo = PgUserRepo {
        pool: fx.pool.clone(),
    };
    let user = repo.insert("13800000021", "hash").await.unwrap();
    let found = repo.find_by_id(user.id).await.unwrap().unwrap();
    assert_eq!(found, user);
}

#[tokio::test]
async fn find_by_id_returns_none_for_missing() {
    let fx = setup_pg().await;
    let repo = PgUserRepo {
        pool: fx.pool.clone(),
    };
    assert!(repo.find_by_id(99999).await.unwrap().is_none());
}

#[tokio::test]
async fn duplicate_phone_rejected_with_unique_violation() {
    let fx = setup_pg().await;
    let repo = PgUserRepo {
        pool: fx.pool.clone(),
    };
    repo.insert("13800000022", "hash1").await.unwrap();
    let err = repo.insert("13800000022", "hash2").await.unwrap_err();
    assert!(matches!(
        err,
        lol_web_server::domain::RepoError::UniqueViolation
    ));
}

#[tokio::test]
async fn update_password_changes_hash() {
    let fx = setup_pg().await;
    let repo = PgUserRepo {
        pool: fx.pool.clone(),
    };
    let user = repo.insert("13800000023", "old_hash").await.unwrap();
    repo.update_password(user.id, "new_hash").await.unwrap();

    let (_, hash) = repo.find_by_phone("13800000023").await.unwrap().unwrap();
    assert_eq!(hash, "new_hash");
}

#[tokio::test]
async fn update_password_for_missing_user_returns_not_found() {
    let fx = setup_pg().await;
    let repo = PgUserRepo {
        pool: fx.pool.clone(),
    };
    let err = repo.update_password(99999, "hash").await.unwrap_err();
    assert!(matches!(err, lol_web_server::domain::RepoError::NotFound));
}
