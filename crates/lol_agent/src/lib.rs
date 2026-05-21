pub mod models;
pub mod systems;

use bevy::prelude::*;
pub use models::*;
pub use systems::*;

pub struct PluginAgentObserver;

impl Plugin for PluginAgentObserver {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_ws_request);
    }
}
