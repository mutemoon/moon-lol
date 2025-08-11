mod animation;
mod camera;
mod character_cache;
mod map;
mod resource;
mod ui;

pub use animation::*;
pub use camera::*;
pub use character_cache::*;
pub use map::*;
pub use resource::*;
pub use ui::*;

use bevy::prelude::*;

pub struct PluginRender;

impl Plugin for PluginRender {
    fn build(&self, app: &mut App) {
        app.add_plugins(PluginCamera);
        app.add_plugins(PluginResource);
        app.add_plugins(PluginMap);
        app.add_plugins(PluginAnimation);
        app.init_resource::<CharacterResourceCache>();
    }
}
