mod ability_resource;
mod bounding;
mod buff;
mod direction;
mod level;
mod pipeline;
mod position;
mod state;

use bevy::app::{App, Plugin};

#[derive(Default)]
pub struct PluginBase;

impl Plugin for PluginBase {
    fn build(&self, _app: &mut App) {}
}
