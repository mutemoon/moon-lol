pub mod models;
pub mod tools;
pub mod systems;

pub use models::*;
pub use tools::*;
pub use systems::*;

use bevy::prelude::*;

pub struct PluginAgent;

impl Plugin for PluginAgent {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActorResource {
            actor: AiActor::default(),
            timer: Timer::from_seconds(5.0, TimerMode::Once),
            phase: ActorPhase::Thinking,
            cached_action: None,
        })
        .add_systems(Update, update_actor);
    }
}
