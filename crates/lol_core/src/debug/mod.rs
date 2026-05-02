pub mod handlers;
pub mod protocol;
pub mod server;

use bevy::prelude::*;
use handlers::ChampionSwitchQueue;

/// Debug panel plugin.
/// Starts a WebSocket server on the given port so the Tauri debug panel
/// can send runtime commands (switch champion, toggle god mode, etc.).
pub struct PluginDebugPanel {
    pub ws_port: u16,
}

impl Plugin for PluginDebugPanel {
    fn build(&self, app: &mut App) {
        let port = self.ws_port;

        app.insert_resource(ChampionSwitchQueue::default());
        app.insert_resource(GlobalDebugState::default());

        app.add_systems(Startup, move |world: &mut World| {
            server::start(world, port);
            server::send_event(world, protocol::WsEvent::game_loaded());
        });

        // app.add_systems(Update, |world: &mut World| {
        //     server::poll_commands(world);
        // });
    }
}

/// Global debug state tracked across commands.
#[derive(Resource, Default)]
pub struct GlobalDebugState {
    pub cooldown_disabled: bool,
    pub god_mode: bool,
    pub paused: bool,
}
