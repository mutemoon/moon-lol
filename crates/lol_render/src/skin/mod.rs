use bevy::prelude::*;

pub mod mesh_shadow;
pub mod skin;

use self::skin::{try_load_config_skin_characters, update_skin_scale};

#[derive(Default)]
pub struct PluginSkin;

impl Plugin for PluginSkin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_skin_scale);
        app.add_systems(Update, try_load_config_skin_characters);
    }
}
