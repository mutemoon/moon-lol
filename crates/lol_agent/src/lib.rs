pub mod driver;
pub mod models;
pub mod rl;
pub mod systems;

use bevy::prelude::*;
pub use driver::*;
pub use models::*;
pub use rl::*;
pub use systems::*;

pub struct PluginAgentObserver;

impl Plugin for PluginAgentObserver {
    fn build(&self, app: &mut App) {
        app.init_non_send::<driver::ScriptRuntimes>();
        app.init_resource::<rl::RlEnvs>();
        app.add_observer(on_command_ws_request);
        app.add_systems(FixedUpdate, drive_script_agents);
    }
}
