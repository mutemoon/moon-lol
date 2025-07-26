// Entities module
mod champions;
mod minion;
mod nexus;

pub use champions::*;
pub use minion::*;
pub use nexus::*;

use bevy::prelude::*;

pub struct PluginEntities;

impl Plugin for PluginEntities {
    fn build(&self, app: &mut App) {
        app.add_plugins((PluginMinion, PluginNexus, PluginChampions));
    }
}
