pub mod ability_resource;
pub mod bounding;
pub mod buff;
pub mod direction;
pub mod level;
pub mod pipeline;
pub mod position;
pub mod state;
use bevy::app::{App, Plugin};

#[derive(Default)]
pub struct PluginBase;

impl Plugin for PluginBase {
    fn build(&self, _app: &mut App) {}
}
