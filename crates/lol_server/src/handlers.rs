use bevy::prelude::*;

use crate::protocol::WsResponse;

/// Dispatch a WS command by broadcasting a generic Bevy Event.
pub fn dispatch(world: &mut World, id: u64, cmd: String, params: serde_json::Value) -> WsResponse {
    lol_rpc::dispatch(world, id, &cmd, params)
}
