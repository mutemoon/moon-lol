pub mod ability_resource;
pub mod bounding;
pub mod buff;
pub mod direction;
pub mod gold;
pub mod level;
pub mod pipeline;
pub mod position;
pub mod state;
pub mod stats;
use bevy::app::{App, Plugin};

#[derive(Default)]
pub struct PluginBase;

impl Plugin for PluginBase {
    fn build(&self, app: &mut App) {
        app.add_plugins(gold::PluginGold);
        app.add_plugins(stats::PluginChampionStats);
    }
}
