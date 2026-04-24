pub mod animation;
pub mod camera;
pub mod controller;
pub mod error;
pub mod loaders;
pub mod map;
pub mod particle;
pub mod resource;
pub mod shader;
pub mod skin;
pub mod test_render;
pub mod ui;

use bevy::prelude::{App, Plugin};
use camera::PluginCamera;
use controller::PluginController;
use map::PluginRenderMap;
use resource::PluginRenderResource;
use skin::PluginSkin;

#[derive(Default)]
pub struct PluginRender;

impl Plugin for PluginRender {
    fn build(&self, app: &mut App) {
        // app.add_plugins(PluginAnimation);
        app.add_plugins(PluginCamera);
        app.add_plugins(PluginController);
        app.add_plugins(PluginRenderMap);
        // app.add_plugins(PluginParticle);
        app.add_plugins(PluginRenderResource);
        app.add_plugins(PluginSkin);
        // app.add_plugins(PluginUI);
    }
}
