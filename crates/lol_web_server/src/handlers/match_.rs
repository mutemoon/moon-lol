//! Match 路由：列表 / 查询 / 事件流 / 停止对局。

use axum::extract::{Path, Query, State};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::header::AUTHORIZATION;
use axum::http::HeaderMap;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use uuid::Uuid;

use super::response::ApiResponse;
use super::{AppState, AuthUser, JwtClaims};
use crate::service::match_supervisor::{get_broadcasters, SpectatorMessage};

#[derive(Deserialize)]
pub struct ListMatchesQuery {
    pub status: Option<String>,
}

pub async fn list_matches(
    auth: AuthUser,
    State(s): State<AppState>,
    Query(q): Query<ListMatchesQuery>,
) -> ApiResponse<Vec<crate::domain::match_::Match>> {
    if let Some(status_str) = q.status {
        if let Some(status) = crate::domain::match_::MatchStatus::from_str(&status_str) {
            return match s.match_service.list_by_status(status).await {
                Ok(list) => ApiResponse::ok(list),
                Err(e) => ApiResponse::from_error(e),
            };
        }
    }
    match s.match_service.list_mine(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn get_match(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<crate::domain::match_::Match> {
    match s.match_service.get(auth.user_id, id).await {
        Ok(m) => ApiResponse::ok(m),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Deserialize)]
pub struct GetEventsQuery {
    pub from_seq: Option<i32>,
    pub limit: Option<i64>,
}

pub async fn get_match_events(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<GetEventsQuery>,
) -> ApiResponse<Vec<crate::domain::match_::MatchEvent>> {
    match s
        .match_service
        .get_events(
            auth.user_id,
            id,
            q.from_seq.unwrap_or(0),
            q.limit.unwrap_or(100),
        )
        .await
    {
        Ok(events) => ApiResponse::ok(events),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn stop_match(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.local_game_service.stop(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Deserialize)]
pub struct WsEventsQuery {
    pub token: Option<String>,
    pub from_seq: Option<i32>,
}

pub async fn get_match_events_ws(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<WsEventsQuery>,
) -> impl axum::response::IntoResponse {
    // 1. 从 Header 或 Query 参数中获取并验证 Token
    let token = if let Some(auth_header) = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        auth_header.strip_prefix("Bearer ").unwrap_or(auth_header).to_string()
    } else if let Some(token_param) = q.token.clone() {
        token_param
    } else {
        return axum::response::Response::builder()
            .status(axum::http::StatusCode::UNAUTHORIZED)
            .body(axum::body::Body::empty())
            .unwrap();
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "moon-lol-secret-key-12345".into());
    let claims = match decode::<JwtClaims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data.claims,
        Err(_) => {
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::UNAUTHORIZED)
                .body(axum::body::Body::empty())
                .unwrap();
        }
    };

    // 2. 校验对局是否存在且可访问
    let user_id = claims.user_id;
    match s.match_service.get(user_id, id).await {
        Ok(_) => {}
        Err(_) => {
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::FORBIDDEN)
                .body(axum::body::Body::empty())
                .unwrap();
        }
    }

    // 3. 升级 WS 连接
    ws.on_upgrade(move |socket| handle_ws_events(socket, s, id, user_id, q.from_seq.unwrap_or(0)))
}

async fn handle_ws_events(
    mut socket: WebSocket,
    state: AppState,
    match_id: Uuid,
    user_id: i32,
    from_seq: i32,
) {
    // 1. 优先订阅广播接收器，防止漏掉期间的实时事件
    let rx_opt = {
        let lock = get_broadcasters().lock().unwrap();
        lock.get(&match_id).map(|tx| tx.subscribe())
    };

    // 2. 查询并发送从 `from_seq` 开始的所有历史数据库事件
    let mut last_seq = from_seq - 1;
    let limit = 200;
    let mut current_from = from_seq;
    loop {
        match state.match_service.get_events(user_id, match_id, current_from, limit).await {
            Ok(events) => {
                if events.is_empty() {
                    break;
                }
                for ev in events {
                    last_seq = last_seq.max(ev.seq);
                    let msg = SpectatorMessage::Event(ev);
                    if let Ok(txt) = serde_json::to_string(&msg) {
                        if socket.send(Message::Text(txt.into())).await.is_err() {
                            return;
                        }
                    }
                }
                current_from = last_seq + 1;
            }
            Err(_) => break,
        }
    }

    // 3. 如果当前对局已结束（没有活跃广播器），则发送 close 并关闭连接
    let mut rx = match rx_opt {
        Some(r) => r,
        None => {
            let close_msg = SpectatorMessage::Close {
                reason: "match is not in progress".to_string(),
            };
            if let Ok(txt) = serde_json::to_string(&close_msg) {
                let _ = socket.send(Message::Text(txt.into())).await;
            }
            return;
        }
    };

    // 4. 持续接收广播事件并推送给 WS 客户端，过滤已发送的历史事件
    while let Ok(msg) = rx.recv().await {
        match &msg {
            SpectatorMessage::Event(ev) => {
                if ev.seq <= last_seq {
                    continue;
                }
                last_seq = ev.seq;
            }
            _ => {}
        }

        if let Ok(txt) = serde_json::to_string(&msg) {
            if socket.send(Message::Text(txt.into())).await.is_err() {
                break;
            }
        }

        if let SpectatorMessage::Close { .. } = msg {
            break;
        }
    }
}
