//! Handler 层：axum 路由薄层（参数解析 → service → 序列化）。
//!
//! 对应 docs/API_DESIGN.md §3 Web Server 接口。
//! 鉴权：HTTP 走 JWT Bearer（Claims extractor）。
//!
//! 共享基础设施（`AppState` / `AuthUser` / `ApiResponse` 等）与
//! 路由构建 `create_router` 定义在本文件；各业务路由按域拆分到
//! 同名子模块（auth / config / agent / room / match_ / ... ）。

use std::sync::Arc;

use axum::Router;
use axum::http::request::Parts;
use axum::routing::{delete, get, patch, post};

use crate::domain::ServiceError;
use crate::service::*;

pub mod admin;
pub mod agent;
pub mod agent_snapshot;
pub mod auth;
pub mod community;
pub mod config;
pub mod essence;
pub mod local_game;
pub mod match_;
pub mod rank;
pub mod response;
pub mod room;
pub mod scenario;
pub mod spawn_preset;
pub mod subscription;

// 重新导出测试与调用方依赖的公共类型，保持 `handlers::Xxx` 路径不变。
pub use agent::UpdateVisibilityRequest;
pub use auth::{
    AuthResponse, AuthUserDto, CodeLoginRequest, LoginRequest, RegisterRequest,
    ResetPasswordRequest,
};
pub use community::{BrowseQuery, ForkRequest};
pub use essence::{CheckInDto, TransactionsQuery};
pub use match_::{GetEventsQuery, ListMatchesQuery};
pub use rank::{LeaderboardQuery, RankEnqueueRequest};
pub use response::{ApiError, ApiResponse};
pub use room::{AddSlotRequest, CreateRoomRequest, JoinByCodeRequest, StartRoomResponse};
pub use subscription::SubscribeRequest;

// ── AppState ──

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<dyn UserService>,
    pub config_service: Arc<dyn ConfigService>,
    pub spawn_preset_service: Arc<dyn SpawnPresetService>,
    pub agent_service: Arc<dyn AgentService>,
    pub agent_snapshot_service: Arc<dyn AgentSnapshotService>,
    pub scenario_service: Arc<dyn ScenarioService>,
    pub room_service: Arc<dyn RoomService>,
    pub match_service: Arc<dyn MatchService>,
    pub rank_service: Arc<dyn RankService>,
    pub essence_service: Arc<dyn EssenceService>,
    pub subscription_service: Arc<dyn SubscriptionService>,
    pub community_service: Arc<dyn CommunityService>,
    pub local_game_service: Arc<dyn LocalGameService>,
    pub admin_service: Arc<dyn AdminService>,
    pub log_service: Arc<dyn LogService>,
}

// ── JWT Claims Extractor ──

pub struct AuthUser {
    pub user_id: i32,
}

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiResponse<()>;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        use axum::http::header::AUTHORIZATION;
        use jsonwebtoken::{DecodingKey, Validation, decode};

        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiResponse::<()>::from_error(ServiceError::Unauthorized))?;

        let token = if let Some(stripped) = auth_header.strip_prefix("Bearer ") {
            stripped
        } else {
            auth_header
        };

        let secret =
            std::env::var("JWT_SECRET").unwrap_or_else(|_| "moon-lol-secret-key-12345".into());

        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| ApiResponse::<()>::from_error(ServiceError::Unauthorized))?;

        Ok(AuthUser {
            user_id: token_data.claims.user_id,
        })
    }
}

/// JWT claims（与 domain::auth::Claims 兼容，用于 extractor）。
#[derive(serde::Serialize, serde::Deserialize)]
pub struct JwtClaims {
    pub user_id: i32,
    pub exp: usize,
}

// ── 路由构建 ──

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Auth
        .route("/api/auth/register", post(auth::auth_register))
        .route("/api/auth/login", post(auth::auth_login))
        .route("/api/auth/code-login", post(auth::auth_code_login))
        .route("/api/auth/reset-password", post(auth::auth_reset_password))
        .route("/api/auth/me", get(auth::auth_me))
        // AI Config
        .route(
            "/api/config",
            get(config::get_config).post(config::set_config),
        )
        // Spawn Presets
        .route(
            "/api/spawn-presets",
            get(spawn_preset::list_spawn_presets).post(spawn_preset::create_spawn_preset),
        )
        .route(
            "/api/spawn-presets/:id",
            get(spawn_preset::get_spawn_preset)
                .put(spawn_preset::update_spawn_preset)
                .delete(spawn_preset::delete_spawn_preset),
        )
        // Agents
        .route(
            "/api/agents",
            get(agent::list_agents).post(agent::create_agent),
        )
        .route(
            "/api/agents/:id",
            get(agent::get_agent)
                .put(agent::update_agent)
                .delete(agent::delete_agent),
        )
        .route(
            "/api/agents/:id/visibility",
            patch(agent::update_agent_visibility),
        )
        .route(
            "/api/agents/:id/publish",
            post(agent_snapshot::publish_snapshot),
        )
        .route(
            "/api/agents/:id/snapshots",
            get(agent_snapshot::list_snapshots),
        )
        .route("/api/agents/community", get(community::browse_community))
        .route("/api/agents/:id/fork", post(community::fork_agent))
        .route(
            "/api/agents/:id/pull-upstream",
            post(community::pull_upstream_agent),
        )
        // Scenarios
        .route(
            "/api/scenarios",
            get(scenario::list_scenarios).post(scenario::create_scenario),
        )
        .route(
            "/api/scenarios/:id",
            get(scenario::get_scenario)
                .put(scenario::update_scenario)
                .delete(scenario::delete_scenario),
        )
        .route(
            "/api/scenarios/:id/win-condition",
            get(scenario::get_win_condition).put(scenario::save_win_condition),
        )
        // Rooms
        .route(
            "/api/rooms",
            get(room::list_my_rooms).post(room::create_room),
        )
        .route("/api/rooms/lobby", get(room::list_lobby_rooms))
        .route("/api/rooms/join-by-code", post(room::join_room_by_code))
        .route(
            "/api/rooms/:id",
            get(room::get_room)
                .delete(room::dissolve_room)
                .patch(room::update_room_constraints),
        )
        .route("/api/rooms/:id/join", post(room::join_room))
        .route("/api/rooms/:id/leave", post(room::leave_room))
        .route(
            "/api/rooms/:id/agents",
            get(room::list_room_slots).post(room::add_room_slot),
        )
        .route(
            "/api/rooms/:id/agents/:slot_id",
            delete(room::remove_room_slot),
        )
        .route("/api/rooms/:id/start", post(room::start_room_match))
        // Matches
        .route("/api/matches", get(match_::list_matches))
        .route("/api/matches/:id", get(match_::get_match))
        .route("/api/matches/:id/events", get(match_::get_match_events))
        .route("/api/matches/:id/stop", post(match_::stop_match))
        // Local Game
        .route("/api/local/start", post(local_game::local_start))
        .route("/api/local/stop", post(local_game::local_stop))
        // Rank
        .route("/api/rank/queue", post(rank::rank_enqueue))
        .route("/api/rank/queue/status", get(rank::rank_queue_status))
        .route("/api/rank/leaderboard", get(rank::rank_leaderboard))
        .route("/api/rank/seasons/current", get(rank::current_season))
        // Essence
        .route("/api/essence/balance", get(essence::essence_balance))
        .route("/api/essence/check-in", post(essence::essence_check_in))
        .route(
            "/api/essence/transactions",
            get(essence::essence_transactions),
        )
        // Subscriptions
        .route(
            "/api/subscriptions",
            get(subscription::get_subscription).post(subscription::subscribe),
        )
        .route("/api/billing/plans", get(subscription::list_plans))
        // Admin
        .route("/api/admin/metrics", get(admin::admin_metrics))
        .route("/api/admin/matches/running", get(admin::admin_running))
        .route(
            "/api/admin/matches/:id/abort",
            post(admin::admin_force_abort),
        )
        .with_state(state)
}
