use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

mod handlers;
mod interfaces;
mod models;
mod services;

use handlers::{create_router, AppState};
use services::{
    ConfigServiceImpl, PresetServiceImpl, ScenarioServiceImpl,
    GameServiceImpl, HistoryServiceImpl, LogServiceImpl, UserServiceImpl
};

#[tokio::main]
async fn main() {
    // Initialize tracing logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    // Connect to PostgreSQL database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/moon_lol".to_string());

    tracing::info!("Connecting to database: {}", database_url);
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL. Please check DATABASE_URL.");

    // Initialize database tables & migrations
    tracing::info!("Initializing database tables...");
    services::init_db(&pool).await.expect("Failed to initialize database tables");

    // Instantiate service implementations
    let state = AppState {
        config_service: Arc::new(ConfigServiceImpl { pool: pool.clone() }),
        preset_service: Arc::new(PresetServiceImpl { pool: pool.clone() }),
        scenario_service: Arc::new(ScenarioServiceImpl { pool: pool.clone() }),
        game_service: Arc::new(GameServiceImpl::new()),
        history_service: Arc::new(HistoryServiceImpl { pool: pool.clone() }),
        log_service: Arc::new(LogServiceImpl),
        user_service: Arc::new(UserServiceImpl { pool: pool.clone() }),
    };

    // Create the router and layer with CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = create_router(state).layer(cors);

    // Bind to port (read from PORT env variable, default to 9090)
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "9090".to_string())
        .parse::<u16>()
        .unwrap_or(9090);
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Server listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
