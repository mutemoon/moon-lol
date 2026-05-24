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

        app.add_systems(Startup, move |world: &mut World| {
            server::start(world, port);
            server::send_event(world, protocol::WsEvent::game_loaded());
        });

        app.add_systems(Update, |world: &mut World| {
            server::poll_commands(world);
        });
    }
}

