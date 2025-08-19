use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy::render::{
    settings::{Backends, RenderCreation, WgpuSettings},
    RenderPlugin,
};
use moon_lol::core::{
    spawn_skin_entity, Attack, CommandAttackCast, ConfigGame, ConfigNavigationGrid, Controller,
    Focus, Health, Movement, PluginGame, Target, Team, WindupConfig,
};
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
            PluginCore.build().disable::<PluginGame>(),
            PluginEntities,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (|mut commands: Commands, q_controller: Query<(Entity, &Controller)>| {
                println!("key pressed A");
                for (entity, _) in q_controller.iter() {
                    println!("attack cast");
                    commands.trigger_targets(CommandAttackCast, entity);
                }
            })
            .run_if(input_just_pressed(KeyCode::KeyA)),
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
    for (_, team, skin) in config_game.legends.clone() {
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
                    value: 600.0,
                    max: 600.0,
                },
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
                team,
                Controller,
                Focus,
                Movement { speed: 325.0 },
                Health {
                    value: 600.0,
                    max: 600.0,
                },
                Attack {
                    range: 100.0,
                    base_attack_speed: 1.0,
                    bonus_attack_speed: 0.0,
                    attack_speed_cap: 2.5,
                    windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
                    windup_modifier: 1.0,
                },
                Target(chaos_entity),
            ))
            .log_components();
    }
}
