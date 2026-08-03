pub mod animation;
pub mod aoe_visual;
pub mod audio;
pub mod controller;
pub mod cursor;
pub mod debug_area;
pub mod debug_missile;
pub mod debug_sphere;
pub mod loaders;
pub mod map;
pub mod navigation;
pub mod skin;
pub mod test_render;
pub mod ui;

// 因为 camera / shader / error 已下沉到 lol_base_render（供 lol_particle 等下游复用以避免循环依赖），
// 所以在此 re-export 保持 crate::camera / crate::shader / crate::error 调用路径稳定。
use animation::PluginAnimation;
use aoe_visual::PluginAoEVisual;
use audio::PluginAudio;
use bevy::prelude::{App, Plugin};
use controller::PluginController;
use cursor::PluginCursor;
use debug_area::PluginDebugArea;
use debug_missile::PluginDebugMissile;
use debug_sphere::PluginDebugSphere;
use lol_base_render::camera::PluginCamera;
pub use lol_base_render::{camera, error, shader};
use map::PluginRenderMap;
use navigation::PluginRenderNavigation;
use skin::PluginSkin;
use ui::PluginUI;

#[derive(Default)]
pub struct PluginRender;

impl Plugin for PluginRender {
    fn build(&self, app: &mut App) {
        league_core::register::init_league_asset(app);
        app.add_plugins(PluginAnimation);
        app.add_plugins(PluginAoEVisual);
        app.add_plugins(PluginAudio);
        app.add_plugins(PluginCamera);
        app.add_plugins(PluginController);
        app.add_plugins(PluginCursor);
        app.add_plugins(PluginDebugArea);
        app.add_plugins(PluginDebugMissile);
        app.add_plugins(PluginDebugSphere);
        app.add_plugins(PluginRenderMap);
        app.add_plugins(PluginRenderNavigation);
        app.add_plugins(PluginSkin);
        app.add_plugins(PluginUI);
    }
}
