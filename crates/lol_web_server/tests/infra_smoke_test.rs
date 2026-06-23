//! 验证 testcontainers + sqlx + schema 的基础设施是否真的可用。
//! 这是所有 repo 集成测试的前提：如果这个测试跑不通，整个测试策略需要重定。

mod common;

use common::setup_pg;
use sqlx::Row;

#[tokio::test]
async fn pg_container_starts_and_schema_loads() {
    let fx = setup_pg().await;

    // 验证 users 表存在并能插入
    sqlx::query("INSERT INTO users (phone, password_hash) VALUES ($1, $2)")
        .bind("13800000001")
        .bind("hash_placeholder")
        .execute(&fx.pool)
        .await
        .expect("插入 users 失败");

    // 验证能读回
    let row = sqlx::query("SELECT id, phone FROM users WHERE phone = $1")
        .bind("13800000001")
        .fetch_one(&fx.pool)
        .await
        .expect("查询 users 失败");
    let phone: String = row.try_get("phone").unwrap();
    assert_eq!(phone, "13800000001");

    // 验证 UUID 主键的表（agent_configs）能插入
    let cfg_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO agent_configs (id, owner_id, name, agent_type) VALUES ($1, $2, $3, $4)",
    )
    .bind(cfg_id)
    .bind(1)
    .bind("激进压制")
    .bind("llm")
    .execute(&fx.pool)
    .await
    .expect("插入 agent_configs 失败");

    // 验证 JSONB 字段能用 sqlx::types::Json
    let row = sqlx::query("SELECT config_json FROM agent_configs WHERE id = $1")
        .bind(cfg_id)
        .fetch_one(&fx.pool)
        .await
        .unwrap();
    let config_json: serde_json::Value = row.try_get("config_json").unwrap();
    assert_eq!(config_json, serde_json::json!({}));
}

#[tokio::test]
async fn unique_constraint_triggers() {
    let fx = setup_pg().await;

    sqlx::query("INSERT INTO users (phone, password_hash) VALUES ($1, $2)")
        .bind("13800000002")
        .bind("hash")
        .execute(&fx.pool)
        .await
        .unwrap();

    // 重复 phone 应失败
    let result = sqlx::query("INSERT INTO users (phone, password_hash) VALUES ($1, $2)")
        .bind("13800000002")
        .bind("hash")
        .execute(&fx.pool)
        .await;
    assert!(result.is_err(), "唯一约束应触发");
}
