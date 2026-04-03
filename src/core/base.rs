mod ability_resource;
mod bounding;
mod buff;
mod direction;
mod level;
mod pipeline;
mod position;
mod state;

pub use ability_resource::*;
use bevy::app::{App, Plugin};
pub use bounding::*;
pub use buff::*;
pub use direction::*;
pub use level::*;
pub use pipeline::*;
pub use position::*;
pub use state::*;

#[derive(Default)]
pub struct PluginBase;

impl Plugin for PluginBase {
    fn build(&self, _app: &mut App) {}
}
