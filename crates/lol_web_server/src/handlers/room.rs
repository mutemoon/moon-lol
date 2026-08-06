//! Room 路由：房间 CRUD / 大厅 / 加入离开 / 槽位 / 开赛。

use axum::Json;
use axum::extract::{Path, State};
use lol_web_protocol::room::{
    AddSlotRequest, CreateRoomRequest, JoinByCodeRequest, Room, RoomAgentSlot, StartRoomResponse,
};
use uuid::Uuid;

use super::response::{ApiResponse, api_error};
use super::{AppState, AuthUser};
use crate::service::LocalStartInput;

pub async fn list_my_rooms(auth: AuthUser, State(s): State<AppState>) -> ApiResponse<Vec<Room>> {
    match s.room_service.list_mine(auth.user_id).await {
        Ok(list) => ApiResponse::ok(list.into_iter().map(Into::into).collect()),
        Err(e) => api_error(e),
    }
}

pub async fn create_room(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(req): Json<CreateRoomRequest>,
) -> ApiResponse<Room> {
    let constraints: crate::domain::room::RoomConstraints = req.constraints.into();
    match s
        .room_service
        .create(auth.user_id, req.name, constraints)
        .await
    {
        Ok(r) => ApiResponse::ok(r.into()),
        Err(e) => api_error(e),
    }
}

pub async fn list_lobby_rooms(
    _auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<Vec<Room>> {
    match s.room_service.list_lobby().await {
        Ok(list) => ApiResponse::ok(list.into_iter().map(Into::into).collect()),
        Err(e) => api_error(e),
    }
}

pub async fn join_room_by_code(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(req): Json<JoinByCodeRequest>,
) -> ApiResponse<Room> {
    match s.room_service.join_by_code(auth.user_id, &req.code).await {
        Ok(r) => ApiResponse::ok(r.into()),
        Err(e) => api_error(e),
    }
}

pub async fn get_room(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Room> {
    match s.room_service.get(auth.user_id, id).await {
        Ok(r) => ApiResponse::ok(r.into()),
        Err(e) => api_error(e),
    }
}

pub async fn dissolve_room(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.room_service.dissolve(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}

pub async fn update_room_constraints(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(constraints): Json<lol_web_protocol::room::RoomConstraints>,
) -> ApiResponse<()> {
    let domain_constraints: crate::domain::room::RoomConstraints = constraints.into();
    match s
        .room_service
        .update_constraints(auth.user_id, id, domain_constraints)
        .await
    {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}

pub async fn join_room(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.room_service.join(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}

pub async fn leave_room(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<()> {
    match s.room_service.leave(auth.user_id, id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}

pub async fn list_room_slots(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResponse<Vec<RoomAgentSlot>> {
    match s.room_service.list_slots(auth.user_id, id).await {
        Ok(list) => ApiResponse::ok(list.into_iter().map(Into::into).collect()),
        Err(e) => api_error(e),
    }
}

pub async fn add_room_slot(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AddSlotRequest>,
) -> ApiResponse<RoomAgentSlot> {
    let domain_team: crate::domain::spawn_preset::Team = req.team.into();
    match s
        .room_service
        .add_slot(auth.user_id, id, req.agent_id, domain_team)
        .await
    {
        Ok(slot) => ApiResponse::ok(slot.into()),
        Err(e) => api_error(e),
    }
}

pub async fn remove_room_slot(
    auth: AuthUser,
    State(s): State<AppState>,
    Path((id, slot_id)): Path<(Uuid, Uuid)>,
) -> ApiResponse<()> {
    match s.room_service.remove_slot(auth.user_id, id, slot_id).await {
        Ok(_) => ApiResponse::ok(()),
        Err(e) => api_error(e),
    }
}

pub async fn start_room_match(
    auth: AuthUser,
    State(s): State<AppState>,
    Path(_id): Path<Uuid>,
) -> ApiResponse<StartRoomResponse> {
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
        Err(e) => api_error(e),
    }
}
