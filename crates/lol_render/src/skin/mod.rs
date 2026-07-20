use bevy::prelude::*;

pub mod mesh_shadow;
pub mod particle;
pub mod skin;

use self::particle::{on_command_character_particle_despawn, on_command_character_particle_spawn};
use self::skin::{try_load_config_skin_characters, update_skin_scale};

#[derive(Default)]
pub struct PluginSkin;

impl Plugin for PluginSkin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_character_particle_despawn);
        app.add_observer(on_command_character_particle_spawn);

        app.add_systems(Update, update_skin_scale);
        app.add_systems(Update, try_load_config_skin_characters);
    }
}
