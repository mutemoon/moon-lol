use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use crate::protocol::WsResponse;

#[derive(Event)]
pub struct CommandWsRequest {
    pub id: u64,
    pub cmd: String,
    pub params: serde_json::Value,
    pub response: Arc<Mutex<Option<WsResponse>>>,
}
