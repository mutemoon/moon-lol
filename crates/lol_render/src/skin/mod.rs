use bevy::prelude::*;

pub mod particle;
pub mod skin;

// mesh_shadow 已下沉到 lol_base_render（供 lol_particle 复用），在此 re-export 保持路径兼容。
pub use lol_base_render::mesh_shadow;

use self::particle::{
    on_command_character_particle_despawn, on_command_character_particle_spawn,
    on_event_attack_end_spawn_hit_particle,
};
use self::skin::{try_load_config_skin_characters, update_skin_scale};

#[derive(Default)]
pub struct PluginSkin;

impl Plugin for PluginSkin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_character_particle_despawn);
        app.add_observer(on_command_character_particle_spawn);
        app.add_observer(on_event_attack_end_spawn_hit_particle);

        app.add_systems(Update, update_skin_scale);
        app.add_systems(Update, try_load_config_skin_characters);
    }
}
