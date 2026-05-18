use bevy::prelude::*;

use crate::events::CommandWsRequest;
use crate::protocol::WsResponse;

/// Dispatch a WS command by broadcasting a generic Bevy Event.
pub fn dispatch(world: &mut World, id: u64, cmd: String, params: serde_json::Value) -> WsResponse {
    let response = std::sync::Arc::new(std::sync::Mutex::new(None));

    world.trigger(CommandWsRequest {
        id,
        cmd: cmd.clone(),
        params,
        response: response.clone(),
    });

    let lock = response.lock().unwrap_or_else(|e| e.into_inner());
    match lock.clone() {
        Some(resp) => resp,
        None => WsResponse::err(
            id,
            format!(
                "无任何插件对网络指令 '{}' 做出响应，可能对应模块未挂载",
                cmd
            ),
        ),
    }
}
