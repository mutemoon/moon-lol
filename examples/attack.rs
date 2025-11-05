use std::time::Duration;

use bevy::prelude::*;
use bevy::render::{
    settings::{Backends, RenderCreation, WgpuSettings},
    RenderPlugin,
};

use bevy::winit::{UpdateMode, WinitSettings};
use lol_config::{ConfigGame, ConfigNavigationGrid};
use lol_core::Team;

use moon_lol::abilities::PluginAbilities;
use moon_lol::core::{spawn_skin_entity, CameraState, Controller, Focus, Health};
use moon_lol::entities::{spawn_fiora, PluginBarrack};
use moon_lol::{core::PluginCore, entities::PluginEntities};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "classic 1v1 fiora".to_string(),
                        resolution: (300.0, 300.0).into(),
                        position: WindowPosition::At((0, 1000).into()),
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        ..default()
                    }),
                    ..default()
                }),
            PluginCore,
            PluginEntities.build().disable::<PluginBarrack>(),
            PluginAbilities,
        ))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Reactive {
                wait: Duration::MAX,           // 不超时，只靠你触发
                react_to_device_events: false, // 不理会输入设备事件
                react_to_user_events: false,
                react_to_window_events: false,
            },
            unfocused_mode: UpdateMode::Reactive {
                wait: Duration::MAX,
                react_to_device_events: false, // 不理会输入设备事件
                react_to_user_events: false,
                react_to_window_events: false,
            },
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            |mut q_camera_state: Query<&mut CameraState, Added<CameraState>>| {
                for mut state in q_camera_state.iter_mut() {
                    // state.scale = 0.1;
                }
            },
        )
        .add_systems(
            FixedUpdate,
            |mut commands: Commands,
             q_t: Query<&AttackTarget>,
             mut res_animation_graph: ResMut<Assets<AnimationGraph>>,
             asset_server: Res<AssetServer>,
             res_navigation_grid: Res<ConfigNavigationGrid>,
             config_game: Res<ConfigGame>| {
                if q_t.single().is_ok() {
                    return;
                }

                for (_, team, skin) in config_game.legends.iter() {
                    let map_center_position = res_navigation_grid.get_map_center_position();

                    let target = spawn_skin_entity(
                        &mut commands,
                        &mut res_animation_graph,
                        &asset_server,
                        Transform::from_translation(map_center_position + vec3(100.0, 0.0, -100.0)),
                        &skin,
                    );

                    spawn_fiora(&mut commands, target);

                    commands.entity(target).insert((
                        Team::Chaos,
                        Health {
                            value: 6000.0,
                            max: 6000.0,
                        },
                        AttackTarget,
                    ));
                }
            },
        )
        .run();
}

#[derive(Component)]
struct AttackTarget;

pub fn setup(
    mut commands: Commands,
    res_navigation_grid: Res<ConfigNavigationGrid>,
    mut res_animation_graph: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    config_game: Res<ConfigGame>,
) {
    for (_, team, skin) in config_game.legends.iter() {
        let map_center_position = res_navigation_grid.get_map_center_position();

        let entity = spawn_skin_entity(
            &mut commands,
            &mut res_animation_graph,
            &asset_server,
            Transform::from_translation(map_center_position + vec3(-100.0, 0.0, 100.0)),
            &skin,
        );

        spawn_fiora(&mut commands, entity);

        commands.entity(entity).insert((
            team.clone(),
            Controller::default(),
            Focus,
            Pickable::IGNORE,
        ));
    }
}
