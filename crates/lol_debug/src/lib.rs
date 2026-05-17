pub mod handlers;
pub mod protocol;
pub mod server;

use bevy::prelude::*;

pub struct PluginDebugPanel {
    pub ws_port: u16,
    pub log_receiver: async_channel::Receiver<String>,
}

impl Plugin for PluginDebugPanel {
    fn build(&self, app: &mut App) {
        let port = self.ws_port;

        app.insert_resource(GlobalDebugState::default());
        app.insert_resource(LogReceiver(self.log_receiver.clone()));

        app.add_observer(on_event_agent_decision);

        app.add_systems(Startup, move |world: &mut World| {
            server::start(world, port);
            server::send_event(world, protocol::WsEvent::game_loaded());
        });

        app.add_systems(
            Update,
            (forward_logs, |world: &mut World| {
                server::poll_commands(world);
            }),
        );
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

/// Drain the log bridge channel and forward entries to the WS debug panel.
fn forward_logs(world: &mut World) {
    let rx = world.get_resource::<LogReceiver>().map(|r| r.0.clone());

    let Some(rx) = rx else {
        return;
    };

    while let Ok(msg) = rx.try_recv() {
        let level = if msg.contains("ERROR") || msg.contains(" ERROR ") {
            "error"
        } else if msg.contains("WARN") || msg.contains(" WARN ") {
            "warn"
        } else {
            "info"
        };
        server::send_event(world, protocol::WsEvent::log(level, msg));
    }
}

/// Global debug state tracked across commands.
#[derive(Resource, Default)]
pub struct GlobalDebugState {
    pub cooldown_disabled: bool,
    pub god_mode: bool,
    pub paused: bool,
}

#[derive(Resource)]
pub struct LogReceiver(pub async_channel::Receiver<String>);
