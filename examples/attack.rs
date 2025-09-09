use bevy::prelude::*;
use bevy::render::{
    settings::{Backends, RenderCreation, WgpuSettings},
    RenderPlugin,
};

use lol_config::{ConfigGame, ConfigNavigationGrid};
use lol_core::Team;
use moon_lol::core::{
    spawn_skin_entity, Attack, CameraState, Controller, Focus, Health, Movement, PluginGame,
};
use moon_lol::entities::{Fiora, PluginBarrack};
use moon_lol::{core::PluginCore, entities::PluginEntities, logging::PluginLogging};

fn main() {
    App::new()
        .add_plugins((
            PluginLogging,
            DefaultPlugins
                .build()
                .disable::<bevy::log::LogPlugin>()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "classic 1v1 fiora".to_string(),
                        resolution: (300.0, 300.0).into(),
                        position: WindowPosition::At((0, 0).into()),
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
            PluginCore.build().disable::<PluginGame>(),
            PluginEntities.build().disable::<PluginBarrack>(),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            |mut q_camera_state: Query<&mut CameraState, Added<CameraState>>| {
                for mut state in q_camera_state.iter_mut() {
                    state.scale = 0.1;
                }
            },
        )
        .run();
}

pub fn setup(
    mut commands: Commands,
    res_navigation_grid: Res<ConfigNavigationGrid>,
    mut res_animation_graph: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    config_game: Res<ConfigGame>,
) {
    for (_, team, skin) in config_game.legends.iter() {
        let map_center_position = res_navigation_grid.get_map_center_position();

        let chaos_entity = spawn_skin_entity(
            &mut commands,
            &mut res_animation_graph,
            &asset_server,
            Transform::from_translation(map_center_position + vec3(100.0, 0.0, -100.0)),
            &skin,
        );

        commands
            .entity(chaos_entity)
            .insert((
                Team::Chaos,
                Movement { speed: 325.0 },
                Health {
                    value: 6000.0,
                    max: 6000.0,
                },
                Fiora,
            ))
            .log_components();

        let entity = spawn_skin_entity(
            &mut commands,
            &mut res_animation_graph,
            &asset_server,
            Transform::from_translation(map_center_position + vec3(-100.0, 0.0, 100.0)),
            &skin,
        );

        commands
            .entity(entity)
            .insert((
                team.clone(),
                Controller,
                Focus,
                Movement { speed: 325.0 },
                Health {
                    value: 600.0,
                    max: 600.0,
                },
                Attack::new(150.0, 0.2, 1.45),
                Fiora,
            ))
            .log_components();
    }
}
