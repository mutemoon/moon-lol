pub mod animation;
pub mod camera;
pub mod controller;
pub mod debug_area;
pub mod debug_missile;
pub mod debug_sphere;
pub mod error;
pub mod loaders;
pub mod map;
pub mod navigation;
pub mod particle;
pub mod resource;
pub mod shader;
pub mod skin;
pub mod test_render;
pub mod ui;

use animation::PluginAnimation;
use bevy::prelude::{App, Plugin};
use camera::PluginCamera;
use controller::PluginController;
use debug_area::PluginDebugArea;
use debug_missile::PluginDebugMissile;
use debug_sphere::PluginDebugSphere;
use map::PluginRenderMap;
use navigation::PluginRenderNavigation;
use resource::PluginRenderResource;
use skin::PluginSkin;

#[derive(Default)]
pub struct PluginRender;

impl Plugin for PluginRender {
    fn build(&self, app: &mut App) {
        app.add_plugins(PluginAnimation);
        app.add_plugins(PluginCamera);
        app.add_plugins(PluginController);
        app.add_plugins(PluginDebugArea);
        app.add_plugins(PluginDebugMissile);
        app.add_plugins(PluginDebugSphere);
        app.add_plugins(PluginRenderMap);
        app.add_plugins(PluginRenderNavigation);
        // app.add_plugins(PluginParticle);
        app.add_plugins(PluginRenderResource);
        app.add_plugins(PluginSkin);
        // app.add_plugins(PluginUI);
    }
}
