use bevy::{
    prelude::*,
    render::{
        settings::{
            Backends, InstanceFlags, RenderCreation, WgpuFeatures, WgpuLimits, WgpuSettings,
        },
        RenderPlugin,
    },
};
use moon_lol::{
    classic::PluginClassic,
    combat::*,
    config::NEXUS_BLUE_POSITION,
    controller::Controller,
    entities::Fiora,
    game::GameState,
    logging::PluginLogging,
    render::{Focus, PluginEntities, PluginRender},
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
            // PluginCombat,
            // PluginEntities,
            PluginRender,
            // PluginClassic,
        ))
        .add_systems(OnEnter(GameState::Setup), setup)
        .run();
}

pub fn setup(mut commands: Commands) {
    // commands.spawn((
    //     Fiora,
    //     Controller,
    //     Focus,
    //     Transform::from_xyz(
    //         NEXUS_BLUE_POSITION.x + 500.0,
    //         NEXUS_BLUE_POSITION.y + 500.0,
    //         88.0,
    //     ),
    // ));
}
