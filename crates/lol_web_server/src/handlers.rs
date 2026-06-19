use std::sync::Arc;
use axum::{
    extract::{Path, State, WebSocketUpgrade, ws::{WebSocket, Message}, FromRequestParts},
    http::{StatusCode, request::Parts, header::AUTHORIZATION},
    response::IntoResponse,
    routing::{get, post, delete},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::interfaces::{
    ConfigService, PresetService, ScenarioService, GameService, HistoryService, LogService,
    UserService,
};
use crate::models::{
    AiConfig, SpawnPreset, AgentPreset, HeroPreset, FrontAgentConfig,
    GameConfig, QueryLogsParams, RegisterRequest, LoginRequest, ResetPasswordRequest,
};

#[derive(Clone)]
pub struct AppState {
    pub config_service: Arc<dyn ConfigService>,
    pub preset_service: Arc<dyn PresetService>,
    pub scenario_service: Arc<dyn ScenarioService>,
    pub game_service: Arc<dyn GameService>,
    pub history_service: Arc<dyn HistoryService>,
    pub log_service: Arc<dyn LogService>,
    pub user_service: Arc<dyn UserService>,
}

pub struct Claims {
    pub user_id: i32,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization Header".to_string()))?;

        let token = if auth_header.starts_with("Bearer ") {
            &auth_header[7..]
        } else {
            auth_header
        };

        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "moon-lol-secret-key-12345".to_string());
        let token_data = decode::<crate::services::JwtClaims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid Token: {e}")))?;

        Ok(Claims {
            user_id: token_data.claims.user_id,
        })
    }
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // AI Config
        .route("/api/config", get(get_ai_config).post(set_ai_config))
        
        // Spawn Presets
        .route("/api/presets/spawn", get(list_spawn_presets).post(save_spawn_preset))
        .route("/api/presets/spawn/:name", delete(delete_spawn_preset))
        
        // Agent Presets
        .route("/api/presets/agent", get(list_agent_presets).post(save_agent_preset))
        .route("/api/presets/agent/:name", delete(delete_agent_preset))
        
        // Hero Presets
        .route("/api/presets/hero", get(list_hero_presets).post(save_hero_preset))
        .route("/api/presets/hero/:name", delete(delete_hero_preset))
        
        // Custom Scenarios
        .route("/api/scenarios", get(list_custom_scenarios))
        .route("/api/scenarios/:name", get(load_custom_scenario).post(save_custom_scenario).delete(delete_custom_scenario))
        .route("/api/scenarios/:name/win", get(load_scenario_win_condition).post(save_scenario_win_condition))
        
        // Game Control
        .route("/api/game/start", post(start_game))
        .route("/api/game/stop", post(stop_game))
        
        // Game Histories
        .route("/api/histories", get(list_game_histories))
        .route("/api/histories/:datetime", get(get_game_history_detail).delete(delete_game_history))
        
        // Logs Querying
        .route("/api/logs/entities", get(query_log_entities))
        .route("/api/logs/categories", get(query_log_categories))
        .route("/api/logs/query", post(query_logs))
        .route("/api/logs", delete(clear_logs))
        
        // Auth
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/reset-password", post(reset_password))
        
        // WebSocket Proxy Route
        .route("/api/ws", get(ws_proxy_handler))
        .with_state(state)
}

// ── Handler Implementations ──

// AI Config
async fn get_ai_config(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    match state.config_service.get_ai_config(claims.user_id).await {
        Ok(config) => (StatusCode::OK, Json(config)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn set_ai_config(State(state): State<AppState>, claims: Claims, Json(payload): Json<AiConfig>) -> impl IntoResponse {
    match state.config_service.set_ai_config(claims.user_id, payload).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

// Spawn Presets
async fn list_spawn_presets(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    match state.preset_service.list_spawn_presets(claims.user_id).await {
        Ok(presets) => (StatusCode::OK, Json(presets)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn save_spawn_preset(State(state): State<AppState>, claims: Claims, Json(payload): Json<SpawnPreset>) -> impl IntoResponse {
    match state.preset_service.save_spawn_preset(claims.user_id, payload).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn delete_spawn_preset(State(state): State<AppState>, claims: Claims, Path(name): Path<String>) -> impl IntoResponse {
    match state.preset_service.delete_spawn_preset(claims.user_id, &name).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

// Agent Presets
async fn list_agent_presets(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    match state.preset_service.list_agent_presets(claims.user_id).await {
        Ok(presets) => (StatusCode::OK, Json(presets)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn save_agent_preset(State(state): State<AppState>, claims: Claims, Json(payload): Json<AgentPreset>) -> impl IntoResponse {
    match state.preset_service.save_agent_preset(claims.user_id, payload).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn delete_agent_preset(State(state): State<AppState>, claims: Claims, Path(name): Path<String>) -> impl IntoResponse {
    match state.preset_service.delete_agent_preset(claims.user_id, &name).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

// Hero Presets
async fn list_hero_presets(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    match state.preset_service.list_hero_presets(claims.user_id).await {
        Ok(presets) => (StatusCode::OK, Json(presets)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn save_hero_preset(State(state): State<AppState>, claims: Claims, Json(payload): Json<HeroPreset>) -> impl IntoResponse {
    match state.preset_service.save_hero_preset(claims.user_id, payload).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn delete_hero_preset(State(state): State<AppState>, claims: Claims, Path(name): Path<String>) -> impl IntoResponse {
    match state.preset_service.delete_hero_preset(claims.user_id, &name).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

// Custom Scenarios
async fn list_custom_scenarios(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    match state.scenario_service.list_custom_scenarios(claims.user_id).await {
        Ok(scenarios) => (StatusCode::OK, Json(scenarios)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn load_custom_scenario(State(state): State<AppState>, claims: Claims, Path(name): Path<String>) -> impl IntoResponse {
    match state.scenario_service.load_custom_scenario(claims.user_id, &name).await {
        Ok(agents) => (StatusCode::OK, Json(agents)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn save_custom_scenario(State(state): State<AppState>, claims: Claims, Path(name): Path<String>, Json(payload): Json<Vec<FrontAgentConfig>>) -> impl IntoResponse {
    match state.scenario_service.save_custom_scenario(claims.user_id, &name, payload).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn delete_custom_scenario(State(state): State<AppState>, claims: Claims, Path(name): Path<String>) -> impl IntoResponse {
    match state.scenario_service.delete_custom_scenario(claims.user_id, &name).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn load_scenario_win_condition(State(state): State<AppState>, claims: Claims, Path(name): Path<String>) -> impl IntoResponse {
    match state.scenario_service.load_scenario_win_condition(claims.user_id, &name).await {
        Ok(condition) => (StatusCode::OK, Json(condition)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn save_scenario_win_condition(State(state): State<AppState>, claims: Claims, Path(name): Path<String>, Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    match state.scenario_service.save_scenario_win_condition(claims.user_id, &name, payload).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

// Game Control
async fn start_game(State(state): State<AppState>, claims: Claims, Json(payload): Json<GameConfig>) -> impl IntoResponse {
    match state.game_service.start_game(claims.user_id, payload).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn stop_game(State(state): State<AppState>, _claims: Claims) -> impl IntoResponse {
    match state.game_service.stop_game().await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

// Game Histories
async fn list_game_histories(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    match state.history_service.list_game_histories(claims.user_id).await {
        Ok(histories) => (StatusCode::OK, Json(histories)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn get_game_history_detail(State(state): State<AppState>, claims: Claims, Path(datetime): Path<String>) -> impl IntoResponse {
    match state.history_service.get_game_history_detail(claims.user_id, &datetime).await {
        Ok(detail) => (StatusCode::OK, Json(detail)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn delete_game_history(State(state): State<AppState>, claims: Claims, Path(datetime): Path<String>) -> impl IntoResponse {
    match state.history_service.delete_game_history(claims.user_id, &datetime).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

// Logs Querying
async fn query_log_entities(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    match state.log_service.query_log_entities(claims.user_id).await {
        Ok(entities) => (StatusCode::OK, Json(entities)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn query_log_categories(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    match state.log_service.query_log_categories(claims.user_id).await {
        Ok(categories) => (StatusCode::OK, Json(categories)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn query_logs(State(state): State<AppState>, claims: Claims, Json(payload): Json<QueryLogsParams>) -> impl IntoResponse {
    match state.log_service.query_logs(claims.user_id, payload).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

async fn clear_logs(State(state): State<AppState>, claims: Claims) -> impl IntoResponse {
    match state.log_service.clear_logs(claims.user_id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
    }
}

// ── WebSocket Proxy Handler ──

async fn ws_proxy_handler(
    ws_upgrade: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws_upgrade.on_upgrade(move |socket| handle_ws_socket(socket, state))
}

async fn handle_ws_socket(client_socket: WebSocket, state: AppState) {
    // 1. Get port of the active game session
    let game_port = match state.game_service.get_active_game_port().await {
        Ok(Some(port)) => port,
        _ => {
            eprintln!("[WS Proxy] No active game server port found. Terminating proxy session.");
            return;
        }
    };

    // 2. Connect to the Bevy WebSocket Server
    let bevy_ws_url = format!("ws://127.0.0.1:{}", game_port);
    let bevy_ws_connection = match connect_async(&bevy_ws_url).await {
        Ok((stream, _)) => stream,
        Err(err) => {
            eprintln!("[WS Proxy] Failed to connect to Bevy game WS at {}: {:?}", bevy_ws_url, err);
            return;
        }
    };

    println!("[WS Proxy] Session established for Bevy WS at {}", bevy_ws_url);

    // 3. Bidirectional proxy piping
    let (mut client_write, mut client_read) = client_socket.split();
    let (mut bevy_write, mut bevy_read) = bevy_ws_connection.split();

    // Client -> Web Server -> Bevy
    let mut client_to_bevy = tokio::spawn(async move {
        while let Some(Ok(msg)) = client_read.next().await {
            match msg {
                Message::Text(text) => {
                    let bevy_msg = tokio_tungstenite::tungstenite::Message::Text(text.into());
                    if bevy_write.send(bevy_msg).await.is_err() {
                        break;
                    }
                }
                Message::Binary(bin) => {
                    let bevy_msg = tokio_tungstenite::tungstenite::Message::Binary(bin.into());
                    if bevy_write.send(bevy_msg).await.is_err() {
                        break;
                    }
                }
                Message::Close(_) => {
                    let _ = bevy_write.send(tokio_tungstenite::tungstenite::Message::Close(None)).await;
                    break;
                }
                _ => {}
            }
        }
    });

    // Bevy -> Web Server -> Client
    let mut bevy_to_client = tokio::spawn(async move {
        while let Some(Ok(msg)) = bevy_read.next().await {
            use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
            match msg {
                TungsteniteMessage::Text(text) => {
                    if client_write.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                TungsteniteMessage::Binary(bin) => {
                    if client_write.send(Message::Binary(bin.into())).await.is_err() {
                        break;
                    }
                }
                TungsteniteMessage::Close(frame) => {
                    let close_frame = frame.map(|f| axum::extract::ws::CloseFrame {
                        code: f.code.into(),
                        reason: f.reason,
                    });
                    let _ = client_write.send(Message::Close(close_frame)).await;
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait until one of the directions terminates, then cleanup the other
    tokio::select! {
        _ = &mut client_to_bevy => {
            println!("[WS Proxy] Client disconnected. Cleaning up Bevy session.");
            bevy_to_client.abort();
        }
        _ = &mut bevy_to_client => {
            println!("[WS Proxy] Bevy server disconnected. Cleaning up Client session.");
            client_to_bevy.abort();
        }
    }
}

// Auth
async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    match state.user_service.register(&payload.phone, &payload.password, &payload.code).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err).into_response(),
    }
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    match state.user_service.login(&payload.phone, &payload.password).await {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err).into_response(),
    }
}

async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    match state.user_service.reset_password(&payload.phone, &payload.new_password, &payload.code).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err).into_response(),
    }
}

