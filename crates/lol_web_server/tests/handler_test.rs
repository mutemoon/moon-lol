//! Handler 集成测试：使用 axum oneshot 和 mock service 验证路由。

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use lol_web_server::domain::agent::{Agent, AgentInput, AgentType};
use lol_web_server::domain::agent_snapshot::AgentSnapshot;
use lol_web_server::domain::auth::User;
use lol_web_server::domain::essence::{BillingPlan, EssenceTransaction};
use lol_web_server::domain::match_::{Match, MatchEvent, MatchStatus, Winner};
use lol_web_server::domain::model_provider::{ModelProvider, ModelProviderDto, ModelProviderInput};
use lol_web_server::domain::room::{Room, RoomAgentSlot, RoomConstraints};
use lol_web_server::domain::scenario::{Scenario, ScenarioInput};
use lol_web_server::domain::spawn_preset::{SpawnPreset, SpawnPresetInput, Team, Visibility};
use lol_web_server::domain::{ServiceError, ServiceResult};
use lol_web_server::handlers::{ApiResponse, AppState, AuthUserDto, create_router};
use lol_web_server::repository::essence_repo::Subscription;
use lol_web_server::repository::match_repo::{MatchEventInput, MatchInput};
use lol_web_server::repository::rank_repo::{EloRating, RankQueueEntry, Season};
use lol_web_server::service::*;
use tower::util::ServiceExt;
use uuid::Uuid;

// ── Mock Services ──

mockall::mock! {
    pub UserService {}
    #[async_trait]
    impl UserService for UserService {
        async fn register(&self, phone: &str, password: &str, code: &str) -> ServiceResult<AuthResult>;
        async fn login(&self, phone: &str, password: &str) -> ServiceResult<AuthResult>;
        async fn login_with_code(&self, phone: &str, code: &str) -> ServiceResult<AuthResult>;
        async fn reset_password(&self, phone: &str, new_password: &str, code: &str) -> ServiceResult<()>;
        async fn verify_token(&self, token: &str) -> ServiceResult<User>;
        fn jwt_secret(&self) -> &str;
    }
}

mockall::mock! {
    pub SpawnPresetService {}
    #[async_trait]
    impl SpawnPresetService for SpawnPresetService {
        async fn list(&self, user_id: i32) -> ServiceResult<Vec<SpawnPreset>>;
        async fn create(&self, user_id: i32, input: SpawnPresetInput) -> ServiceResult<SpawnPreset>;
        async fn get(&self, user_id: i32, id: Uuid) -> ServiceResult<SpawnPreset>;
        async fn update(&self, user_id: i32, id: Uuid, input: SpawnPresetInput) -> ServiceResult<()>;
        async fn delete(&self, user_id: i32, id: Uuid) -> ServiceResult<()>;
    }
}

mockall::mock! {
    pub AgentService {}
    #[async_trait]
    impl AgentService for AgentService {
        async fn list(&self, owner_id: i32) -> ServiceResult<Vec<Agent>>;
        async fn get(&self, requester_id: i32, id: Uuid) -> ServiceResult<Agent>;
        async fn create(&self, owner_id: i32, input: AgentInput) -> ServiceResult<Agent>;
        async fn update(&self, owner_id: i32, id: Uuid, input: AgentInput) -> ServiceResult<()>;
        async fn update_visibility(&self, owner_id: i32, id: Uuid, vis: Visibility) -> ServiceResult<()>;
        async fn delete(&self, owner_id: i32, id: Uuid) -> ServiceResult<()>;
        async fn fork(&self, requester_id: i32, source_id: Uuid, new_name: Option<String>) -> ServiceResult<Agent>;
    }
}

mockall::mock! {
    pub AgentSnapshotService {}
    #[async_trait]
    impl AgentSnapshotService for AgentSnapshotService {
        async fn publish(&self, user_id: i32, agent_id: Uuid, config_freeze: serde_json::Value) -> ServiceResult<AgentSnapshot>;
        async fn list_by_agent(&self, agent_id: Uuid) -> ServiceResult<Vec<AgentSnapshot>>;
        async fn find_by_id(&self, id: Uuid) -> ServiceResult<Option<AgentSnapshot>>;
        async fn find_latest(&self, agent_id: Uuid) -> ServiceResult<Option<AgentSnapshot>>;
    }
}

mockall::mock! {
    pub ScenarioService {}
    #[async_trait]
    impl ScenarioService for ScenarioService {
        async fn list(&self, user_id: i32) -> ServiceResult<Vec<Scenario>>;
        async fn create(&self, user_id: i32, input: ScenarioInput) -> ServiceResult<Scenario>;
        async fn get(&self, user_id: i32, id: Uuid) -> ServiceResult<Scenario>;
        async fn update(&self, user_id: i32, id: Uuid, input: ScenarioInput) -> ServiceResult<()>;
        async fn delete(&self, user_id: i32, id: Uuid) -> ServiceResult<()>;
        async fn get_win_condition(&self, user_id: i32, id: Uuid) -> ServiceResult<Option<serde_json::Value>>;
        async fn save_win_condition(&self, user_id: i32, id: Uuid, condition: serde_json::Value) -> ServiceResult<()>;
    }
}

mockall::mock! {
    pub RoomService {}
    #[async_trait]
    impl RoomService for RoomService {
        async fn list_mine(&self, owner_id: i32) -> ServiceResult<Vec<Room>>;
        async fn list_lobby(&self) -> ServiceResult<Vec<Room>>;
        async fn create(&self, owner_id: i32, name: String, constraints: RoomConstraints) -> ServiceResult<Room>;
        async fn get(&self, requester_id: i32, id: Uuid) -> ServiceResult<Room>;
        async fn join_by_code(&self, user_id: i32, code: &str) -> ServiceResult<Room>;
        async fn dissolve(&self, owner_id: i32, id: Uuid) -> ServiceResult<()>;
        async fn update_constraints(&self, owner_id: i32, id: Uuid, constraints: RoomConstraints) -> ServiceResult<()>;
        async fn join(&self, user_id: i32, id: Uuid) -> ServiceResult<()>;
        async fn leave(&self, user_id: i32, id: Uuid) -> ServiceResult<()>;
        async fn list_slots(&self, requester_id: i32, id: Uuid) -> ServiceResult<Vec<RoomAgentSlot>>;
        async fn add_slot(&self, requester_id: i32, id: Uuid, agent_id: Uuid, team: Team) -> ServiceResult<RoomAgentSlot>;
        async fn remove_slot(&self, requester_id: i32, id: Uuid, slot_id: Uuid) -> ServiceResult<()>;
    }
}

mockall::mock! {
    pub MatchService {}
    #[async_trait]
    impl MatchService for MatchService {
        async fn create(&self, owner_id: i32, input: MatchInput) -> ServiceResult<Match>;
        async fn get(&self, requester_id: i32, id: Uuid) -> ServiceResult<Match>;
        async fn list_mine(&self, owner_id: i32) -> ServiceResult<Vec<Match>>;
        async fn list_by_status(&self, status: MatchStatus) -> ServiceResult<Vec<Match>>;
        async fn start(&self, requester_id: i32, id: Uuid, bevy_port: i32, ws_port: i32) -> ServiceResult<Match>;
        async fn finish(&self, requester_id: i32, id: Uuid, winner: Winner) -> ServiceResult<Match>;
        async fn abort(&self, requester_id: i32, id: Uuid, reason: String) -> ServiceResult<Match>;
        async fn append_event(&self, requester_id: i32, id: Uuid, event: MatchEventInput) -> ServiceResult<MatchEvent>;
        async fn finish_internal(&self, id: Uuid, winner: Winner) -> ServiceResult<Match>;
        async fn append_event_internal(&self, id: Uuid, event: MatchEventInput) -> ServiceResult<MatchEvent>;
        async fn get_events(&self, requester_id: i32, id: Uuid, from_seq: i32, limit: i64) -> ServiceResult<Vec<MatchEvent>>;
    }
}

mockall::mock! {
    pub RankService {}
    #[async_trait]
    impl RankService for RankService {
        async fn enqueue(&self, user_id: i32, agent_id: Uuid, agent_snapshot_id: Uuid, mode: &str) -> ServiceResult<RankQueueEntry>;
        async fn dequeue(&self, agent_id: Uuid) -> ServiceResult<()>;
        async fn list_my_queue(&self, user_id: i32) -> ServiceResult<Vec<RankQueueEntry>>;
        async fn try_match(&self, entry: &RankQueueEntry) -> ServiceResult<Option<Uuid>>;
        async fn get_elo(&self, agent_id: Uuid, mode: &str) -> ServiceResult<EloRating>;
        async fn record_result(&self, winner_agent_id: Uuid, loser_agent_id: Uuid, mode: &str, outcome: lol_web_server::domain::rank::Outcome) -> ServiceResult<()>;
        async fn leaderboard(&self, mode: &str, limit: i64) -> ServiceResult<Vec<EloRating>>;
        async fn current_season(&self, mode: &str) -> ServiceResult<Season>;
    }
}

mockall::mock! {
    pub EssenceService {}
    #[async_trait]
    impl EssenceService for EssenceService {
        async fn get_balance(&self, user_id: i32) -> ServiceResult<i64>;
        async fn check_in(&self, user_id: i32, date: &str) -> ServiceResult<CheckInResult>;
        async fn deduct(&self, user_id: i32, amount: i64, reason: &str) -> ServiceResult<i64>;
        async fn grant(&self, user_id: i32, amount: i64, reason: &str) -> ServiceResult<i64>;
        async fn get_transactions(&self, user_id: i32, limit: i64, offset: i64) -> ServiceResult<Vec<EssenceTransaction>>;
    }
}

mockall::mock! {
    pub SubscriptionService {}
    #[async_trait]
    impl SubscriptionService for SubscriptionService {
        async fn get_active_plan(&self, user_id: i32) -> ServiceResult<BillingPlan>;
        async fn subscribe(&self, user_id: i32, plan_id: &str) -> ServiceResult<Subscription>;
        async fn deactivate(&self, user_id: i32) -> ServiceResult<()>;
        async fn get_agent_limit(&self, user_id: i32) -> ServiceResult<usize>;
    }
}

mockall::mock! {
    pub CommunityService {}
    #[async_trait]
    impl CommunityService for CommunityService {
        async fn browse_public(&self, sort: lol_web_server::domain::community::CommunitySort, limit: i64) -> ServiceResult<Vec<Agent>>;
        async fn fork(&self, requester_id: i32, source_id: Uuid, new_name: Option<String>) -> ServiceResult<Agent>;
        async fn pull_upstream(&self, requester_id: i32, id: Uuid) -> ServiceResult<Agent>;
    }
}

mockall::mock! {
    pub LocalGameService {}
    #[async_trait]
    impl LocalGameService for LocalGameService {
        async fn start(&self, owner_id: i32, input: LocalStartInput) -> ServiceResult<(Uuid, i32)>;
        async fn stop(&self, owner_id: i32, match_id: Uuid) -> ServiceResult<()>;
        async fn list_processes(&self) -> ServiceResult<Vec<lol_web_server::domain::local_game::ManagedProcess>>;
        async fn cleanup(&self) -> ServiceResult<usize>;
    }
}

mockall::mock! {
    pub AdminService {}
    #[async_trait]
    impl AdminService for AdminService {
        async fn metrics(&self) -> ServiceResult<AdminMetrics>;
        async fn list_running(&self) -> ServiceResult<Vec<Match>>;
        async fn force_abort(&self, match_id: Uuid) -> ServiceResult<()>;
    }
}

mockall::mock! {
    pub LogService {}
    #[async_trait]
    impl LogService for LogService {
        async fn entities(&self) -> ServiceResult<Vec<serde_json::Value>>;
        async fn categories(&self) -> ServiceResult<Vec<serde_json::Value>>;
        async fn logs(&self, params: QueryLogsParams) -> ServiceResult<QueryLogsResult>;
        async fn clear(&self) -> ServiceResult<()>;
    }
}

mockall::mock! {
    pub ModelProviderService {}
    #[async_trait]
    impl ModelProviderService for ModelProviderService {
        async fn list(&self, owner_id: i32) -> ServiceResult<Vec<ModelProviderDto>>;
        async fn create(&self, owner_id: i32, input: ModelProviderInput) -> ServiceResult<ModelProviderDto>;
        async fn update(&self, owner_id: i32, id: Uuid, input: ModelProviderInput) -> ServiceResult<()>;
        async fn delete(&self, owner_id: i32, id: Uuid) -> ServiceResult<()>;
        async fn resolve_for_runtime(&self, provider_id: Uuid, owner_id: i32) -> ServiceResult<Option<ModelProvider>>;
    }
}

// ── Helper structures ──

struct Mocks {
    user: MockUserService,
    spawn: MockSpawnPresetService,
    agent: MockAgentService,
    snapshot: MockAgentSnapshotService,
    scenario: MockScenarioService,
    room: MockRoomService,
    match_: MockMatchService,
    rank: MockRankService,
    essence: MockEssenceService,
    sub: MockSubscriptionService,
    comm: MockCommunityService,
    local: MockLocalGameService,
    admin: MockAdminService,
    log: MockLogService,
    model_provider: MockModelProviderService,
}

impl Mocks {
    fn new() -> Self {
        Self {
            user: MockUserService::new(),
            spawn: MockSpawnPresetService::new(),
            agent: MockAgentService::new(),
            snapshot: MockAgentSnapshotService::new(),
            scenario: MockScenarioService::new(),
            room: MockRoomService::new(),
            match_: MockMatchService::new(),
            rank: MockRankService::new(),
            essence: MockEssenceService::new(),
            sub: MockSubscriptionService::new(),
            comm: MockCommunityService::new(),
            local: MockLocalGameService::new(),
            admin: MockAdminService::new(),
            log: MockLogService::new(),
            model_provider: MockModelProviderService::new(),
        }
    }
}

fn build_test_state<F>(configure: F) -> AppState
where
    F: FnOnce(&mut Mocks),
{
    let mut mocks = Mocks::new();
    configure(&mut mocks);

    AppState {
        user_service: Arc::new(mocks.user),
        spawn_preset_service: Arc::new(mocks.spawn),
        agent_service: Arc::new(mocks.agent),
        agent_snapshot_service: Arc::new(mocks.snapshot),
        scenario_service: Arc::new(mocks.scenario),
        room_service: Arc::new(mocks.room),
        match_service: Arc::new(mocks.match_),
        rank_service: Arc::new(mocks.rank),
        essence_service: Arc::new(mocks.essence),
        subscription_service: Arc::new(mocks.sub),
        community_service: Arc::new(mocks.comm),
        local_game_service: Arc::new(mocks.local),
        admin_service: Arc::new(mocks.admin),
        log_service: Arc::new(mocks.log),
        model_provider_service: Arc::new(mocks.model_provider),
    }
}

fn generate_test_token(user_id: i32) -> String {
    use jsonwebtoken::{EncodingKey, Header, encode};
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;
    let claims = lol_web_server::handlers::JwtClaims {
        user_id,
        exp: exp as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("moon-lol-secret-key-12345".as_bytes()),
    )
    .unwrap()
}

// ── Integration Tests ──

#[tokio::test]
async fn test_auth_register_success() {
    let user_id = 42;
    let token = "register-test-token".to_string();
    let token_clone = token.clone();

    let state = build_test_state(move |mocks| {
        mocks.user.expect_register().returning(move |phone, _, _| {
            Ok(AuthResult {
                user: User {
                    id: user_id,
                    phone: phone.to_string(),
                },
                token: token_clone.clone(),
            })
        });
    });
    let app = create_router(state);

    let req_body = serde_json::json!({
        "phone": "13800000001",
        "password": "password123",
        "code": "111111"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), 2048)
        .await
        .unwrap();
    let res: ApiResponse<lol_web_server::handlers::AuthResponse> =
        serde_json::from_slice(&body_bytes).unwrap();

    assert!(res.data.is_some());
    let data = res.data.unwrap();
    assert_eq!(data.token, "register-test-token");
    assert_eq!(data.user.id, user_id);
}

#[tokio::test]
async fn test_auth_me_requires_jwt() {
    let state = build_test_state(|_| {});
    let app = create_router(state);

    // Request without Authorization header
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/auth/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 Unauthorized
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_me_with_valid_jwt() {
    let user_id = 123;
    let jwt = generate_test_token(user_id);

    let state = build_test_state(move |mocks| {
        mocks.user.expect_verify_token().returning(move |_| {
            Ok(User {
                id: user_id,
                phone: "13800000001".to_string(),
            })
        });
    });
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/auth/me")
                .header("Authorization", format!("Bearer {jwt}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), 2048)
        .await
        .unwrap();
    let res: ApiResponse<AuthUserDto> = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(res.data.unwrap().id, user_id);
}

#[tokio::test]
async fn test_list_agents_success() {
    let user_id = 1;
    let jwt = generate_test_token(user_id);

    let state = build_test_state(move |mocks| {
        mocks
            .agent
            .expect_list()
            .with(mockall::predicate::eq(user_id))
            .returning(|owner_id| {
                Ok(vec![Agent {
                    id: Uuid::new_v4(),
                    owner_id,
                    name: "Fiora Agent".to_string(),
                    champion: "Fiora".to_string(),
                    agent_type: AgentType::Llm,
                    prompt: "prompt".to_string(),
                    model: "model".to_string(),
                    config_json: serde_json::json!({}),
                    visibility: Visibility::Private,
                    forked_from: None,
                    upstream_agent_id: None,
                }])
            });
    });
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/agents")
                .header("Authorization", format!("Bearer {jwt}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), 2048)
        .await
        .unwrap();
    let res: ApiResponse<Vec<Agent>> = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(res.data.unwrap().len(), 1);
}

#[tokio::test]
async fn test_room_join_success() {
    let user_id = 100;
    let jwt = generate_test_token(user_id);
    let room_id = Uuid::new_v4();

    let state = build_test_state(move |mocks| {
        mocks
            .room
            .expect_join()
            .with(
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(room_id),
            )
            .returning(|_, _| Ok(()));
    });
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/rooms/{room_id}/join"))
                .header("Authorization", format!("Bearer {jwt}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_essence_balance_error_mapping() {
    let user_id = 200;
    let jwt = generate_test_token(user_id);

    let state = build_test_state(move |mocks| {
        mocks
            .essence
            .expect_get_balance()
            .with(mockall::predicate::eq(user_id))
            .returning(|_| Err(ServiceError::Forbidden));
    });
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/essence/balance")
                .header("Authorization", format!("Bearer {jwt}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // ServiceError::Forbidden should be mapped to HTTP 403 Forbidden
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
