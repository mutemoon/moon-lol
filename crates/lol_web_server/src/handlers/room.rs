//! Room 路由：房间 CRUD / 大厅 / 加入离开 / 槽位 / 开赛。

use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::response::ApiResponse;
use super::{AppState, AuthUser};
use crate::service::LocalStartInput;

pub async fn list_my_rooms(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<crate::domain::room::Room>> {
    match s.room_service.list_mine(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub constraints: crate::domain::room::RoomConstraints,
}

pub async fn create_room(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(req): Json<CreateRoomRequest>,
) -> ApiResponse<crate::domain::room::Room> {
    match s
        .room_service
        .create(auth.user_id, req.name, req.constraints)
        .await
    {
        Ok(r) => ApiResponse::ok(r),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn list_lobby_rooms(
    _auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<crate::domain::room::Room>> {
    match s.room_service.list_lobby().await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Deserialize)]
pub struct JoinByCodeRequest {
    pub code: String,
}

pub async fn join_room_by_code(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(req): Json<JoinByCodeRequest>,
) -> ApiResponse<crate::domain::room::Room> {
    match s.room_service.join_by_code(auth.user_id, &req.code).await {
        Ok(r) => ApiResponse::ok(r),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn get_room(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<crate::domain::room::Room> {
    match s.room_service.get(auth.user_id, id).await {
        Ok(r) => ApiResponse::ok(r),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn dissolve_room(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.room_service.dissolve(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn update_room_constraints(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(constraints): Json<crate::domain::room::RoomConstraints>,
) -> ApiResponse<()> {
    match s
        .room_service
        .update_constraints(auth.user_id, id, constraints)
        .await
    {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn join_room(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.room_service.join(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn leave_room(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.room_service.leave(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn list_room_slots(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Vec<crate::domain::room::RoomAgentSlot>> {
    match s.room_service.list_slots(auth.user_id, id).await {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Deserialize)]
pub struct AddSlotRequest {
    pub agent_id: Uuid,
    pub team: crate::domain::spawn_preset::Team,
}

pub async fn add_room_slot(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AddSlotRequest>,
) -> ApiResponse<crate::domain::room::RoomAgentSlot> {
    match s
        .room_service
        .add_slot(auth.user_id, id, req.agent_id, req.team)
        .await
    {
        Ok(slot) => ApiResponse::ok(slot),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn remove_room_slot(
    auth: AuthUser,
    State(s): State<AppState>,
    Path((id, slot_id)): Path<(Uuid, Uuid)>,
) -> ApiResponse<()> {
    match s.room_service.remove_slot(auth.user_id, id, slot_id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Serialize)]
pub struct StartRoomResponse {
    pub match_id: Uuid,
    pub ws_port: i32,
}

pub async fn start_room_match(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(_id): Path<Uuid>,
) -> ApiResponse<StartRoomResponse> {
    // 简化：room start 复用 local_game 启动（实际应由 MatchService 编排）
    match s
        .local_game_service
        .start(
            auth.user_id,
            LocalStartInput {
                mode: "room".into(),
                scenario_id: None,
                win_condition: None,
                scenario_agents: Vec::new(),
            },
        )
        .await
    {
        Ok((match_id, port)) => ApiResponse::ok(StartRoomResponse {
            match_id,
            ws_port: port,
        }),
        Err(e) => ApiResponse::from_error(e),
    }
}
