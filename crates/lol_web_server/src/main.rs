//! MoonLOL Web Server 主程序入口。
//!
//! 负责装配所有 Repository, Cache, Service 实例，
//! 构建 AppState 与 Axum 路由，并启动 HTTP 服务器。

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::http::HeaderValue;
use lol_web_server::cache::MokaConfigCache;
use lol_web_server::handlers::{AppState, create_router};
use lol_web_server::repository::*;
use lol_web_server::service::*;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[tokio::main]
async fn main() {
    // 1. 初始化日志
    tracing_subscriber::fmt::init();

    // 2. 加载环境变量与配置
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/moon_lol".to_string());
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "moon-lol-secret-key-12345".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .unwrap_or(8000);

    info!("连接 Postgres 数据库: {}", db_url);
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("连接 Postgres 数据库失败");

    // 3. 初始化持久层 PgXxxRepo
    let user_repo = Arc::new(PgUserRepo { pool: pool.clone() });
    let config_repo = Arc::new(PgConfigRepo { pool: pool.clone() });
    let spawn_preset_repo = Arc::new(PgSpawnPresetRepo { pool: pool.clone() });
    let agent_repo = Arc::new(PgAgentRepo { pool: pool.clone() });
    let agent_snapshot_repo = Arc::new(PgAgentSnapshotRepo { pool: pool.clone() });
    let scenario_repo = Arc::new(PgScenarioRepo { pool: pool.clone() });
    let room_repo = Arc::new(PgRoomRepo { pool: pool.clone() });

    let match_repo = Arc::new(PgMatchRepo { pool: pool.clone() });
    let participant_repo = Arc::new(PgMatchParticipantRepo { pool: pool.clone() });
    let event_repo = Arc::new(PgMatchEventRepo { pool: pool.clone() });

    let essence_repo = Arc::new(PgEssenceRepo { pool: pool.clone() });
    let subscription_repo = Arc::new(PgSubscriptionRepo { pool: pool.clone() });
    let elo_repo = Arc::new(PgEloRepo { pool: pool.clone() });
    let rank_queue_repo = Arc::new(PgRankQueueRepo { pool: pool.clone() });
    let season_repo = Arc::new(PgSeasonRepo { pool: pool.clone() });

    // 4. 初始化缓存层
    let config_cache = Arc::new(MokaConfigCache::new());

    // 5. 初始化服务层
    let user_service = Arc::new(UserServiceImpl::new(user_repo.clone(), jwt_secret));
    let config_service = Arc::new(ConfigServiceImpl::new(config_repo.clone(), config_cache));
    let spawn_preset_service = Arc::new(SpawnPresetServiceImpl::new(spawn_preset_repo.clone()));

    // SubscriptionServiceImpl 同时实现了 SubscriptionService 和 AgentLimitProvider
    let subscription_service = Arc::new(SubscriptionServiceImpl::new(subscription_repo.clone()));

    let agent_service = Arc::new(AgentServiceImpl::new(
        agent_repo.clone(),
        subscription_service.clone(), // 传入 limit_provider
    ));
    let agent_snapshot_service = Arc::new(AgentSnapshotServiceImpl::new(
        agent_snapshot_repo.clone(),
        agent_repo.clone(),
    ));
    let scenario_service = Arc::new(ScenarioServiceImpl::new(scenario_repo.clone()));
    let room_service = Arc::new(RoomServiceImpl::new(room_repo.clone()));

    let match_service = Arc::new(MatchServiceImpl::new(
        match_repo.clone(),
        participant_repo.clone(),
        event_repo.clone(),
    ));

    let process_launcher = Arc::new(CommandProcessLauncher::new());
    let local_game_service = Arc::new(LocalGameServiceImpl::new(
        match_repo.clone(),
        process_launcher.clone(),
        match_service.clone(), // supervisor 用它落库胜负与事件
    ));

    let rank_service = Arc::new(RankServiceImpl::new(
        season_repo.clone(),
        rank_queue_repo.clone(),
        elo_repo.clone(),
        match_service.clone(), // match_service 实现了 RankMatchCreator
    ));

    let essence_service = Arc::new(EssenceServiceImpl::new(essence_repo.clone()));

    // AdminServiceImpl
    let admin_service = Arc::new(AdminServiceImpl::new(
        match_repo.clone(),
        rank_queue_repo.clone(),
        local_game_service.clone(),
    ));

    let db_path = {
        let base = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        base.join(".moon-lol").join("logs").join("debug.db")
    };
    let log_reader = Arc::new(SqliteLogReader::new(db_path));
    let log_service = Arc::new(LogServiceImpl::new(log_reader));

    // 6. 装配 AppState
    let state = AppState {
        user_service,
        config_service,
        spawn_preset_service,
        agent_service,
        agent_snapshot_service,
        scenario_service,
        room_service,
        match_service,
        rank_service,
        essence_service,
        subscription_service,
        community_service: Arc::new(CommunityServiceImpl::new(agent_repo.clone())),
        local_game_service,
        admin_service,
        log_service,
    };

    // 7. 构建 Router 并启动服务器
    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("http://localhost:1420"))
        .allow_methods(Any)
        .allow_headers(Any);

    let router = create_router(state).layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("MoonLOL Web Server 正在启动，监听地址: {}", addr);

    let listener = TcpListener::bind(addr).await.expect("绑定端口失败");
    axum::serve(listener, router)
        .await
        .expect("运行 Web 服务器失败");
}
