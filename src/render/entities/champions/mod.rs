// Champions module
mod fiora;

pub use fiora::*;

use bevy::prelude::*;

pub struct PluginChampions;

impl Plugin for PluginChampions {
    fn build(&self, app: &mut App) {
        app.add_plugins(PluginRenderFiora);
    }
}
