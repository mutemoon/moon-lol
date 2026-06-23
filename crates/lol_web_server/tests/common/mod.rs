//! 测试共享 fixture：testcontainers 真 PG + schema 初始化。
//!
//! 每个 repo 集成测试用 `setup_pg().await` 拿到独立的 PG 容器（已建表），
//! 测试间完全隔离。需要 Docker（本地或 CI）。

use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

/// 持有 PG 容器与连接池。测试结束后 drop 即回收容器。
pub struct PgFixture {
    pub pool: PgPool,
    // 容器必须存活到 fixture 被 drop；保存在这里延长生命周期。
    _container: testcontainers::ContainerAsync<Postgres>,
}

/// 启动一个独立的 PG 容器，执行全量 schema，返回连接池。
///
/// 每次调用启动新容器（端口随机），测试间隔离。首次调用会拉镜像（~100MB），
/// 后续调用复用已拉取的镜像，启动容器 ~1-2 秒。
pub async fn setup_pg() -> PgFixture {
    let container = Postgres::default().start().await.expect("启动 PG 容器失败");

    let host_port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("获取 PG 端口失败");

    let opts = PgConnectOptions::new()
        .host("127.0.0.1")
        .port(host_port)
        .username("postgres")
        .password("postgres")
        .database("postgres");

    // 容器启动后 PG 需要短暂时间就绪，重试连接
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(15))
        .connect_with(opts)
        .await
        .expect("连接 testcontainers PG 失败");

    // 执行全量 schema（sqlx prepared statement 不支持多语句，逐条执行）
    let schema = include_str!("../../migrations/schema.sql");
    for stmt in split_sql_statements(schema) {
        sqlx::query(&stmt)
            .execute(&pool)
            .await
            .unwrap_or_else(|e| panic!("执行 schema 语句失败: {e}\n语句: {stmt}"));
    }

    PgFixture {
        pool,
        _container: container,
    }
}

/// 把多语句 SQL 按分号切分成单条语句。
///
/// schema.sql 里没有包含分号的字符串字面量，简单按 ';' 切分即可。
/// 跳过空语句和纯注释行。
fn split_sql_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        // 跳过纯注释块（以 -- 开头的整段被分号切分后可能残留）
        .filter(|s| {
            !s.lines()
                .all(|line| line.trim().is_empty() || line.trim().starts_with("--"))
        })
        .map(|s| s.to_string())
        .collect()
}
