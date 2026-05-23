pub mod events;
pub mod handlers;
pub mod protocol;
pub mod server;

use bevy::prelude::*;

pub struct PluginServer {
    pub ws_port: u16,
}

impl Plugin for PluginServer {
    fn build(&self, app: &mut App) {
        let port = self.ws_port;

        app.add_observer(on_event_agent_decision);

        app.add_systems(Startup, move |world: &mut World| {
            server::start(world, port);
            server::send_event(world, protocol::WsEvent::game_loaded());
        });

        app.add_systems(Update, |world: &mut World| {
            server::poll_commands(world);
        });
    }
}

/// Bevy Observer: 监听 AI 决策消息事件并转发至 WebSocket 连接
fn on_event_agent_decision(
    event: On<lol_core::action::EventAgentDecision>,
    ch: Res<server::DebugWsChannel>,
) {
    let evt = protocol::WsEvent::agent_update(
        event.observe.clone(),
        event.thinking.clone(),
        event.action.clone(),
    );
    if let Ok(json) = serde_json::to_string(&evt) {
        let _ = ch.out_tx.try_send(json);
    }
}
