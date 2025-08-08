use bevy::{
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
use moon_lol::{
    combat::PluginCombat, entities::PluginEntities, logging::PluginLogging, render::PluginRender,
};

fn main() {
    App::new()
        .add_plugins((
            // PluginLogging, // Add logging first to capture all system logs
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "classic 1v1 fiora".to_string(),
                        // resolution: (300.0, 300.0).into(),
                        // position: WindowPosition::At((0, 1920).into()),
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        // limits: WgpuLimits::downlevel_webgl2_defaults(),
                        ..default()
                    }),
                    ..default()
                }),
            PluginCombat,
            PluginEntities,
            PluginRender,
            PluginLogging,
        ))
        .run();
}
