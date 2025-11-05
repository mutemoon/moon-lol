use bevy::prelude::*;

use lol_config::{ConfigGame, ConfigNavigationGrid};
use lol_core::Team;

use crate::{
    core::{spawn_skin_entity, Controller, Focus, Health},
    entities::spawn_fiora,
};

#[derive(Default)]
pub struct PluginGymEnv;

impl Plugin for PluginGymEnv {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_fiora_player);
        app.add_systems(FixedUpdate, spawn_target);
    }
}

#[derive(Component)]
struct AttackTarget;

fn setup_fiora_player(
    mut commands: Commands,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut res_animation_graph: ResMut<Assets<AnimationGraph>>,
    config_game: Res<ConfigGame>,
    asset_server: Res<AssetServer>,
    grid: Res<ConfigNavigationGrid>,
) {
    virtual_time.set_relative_speed(1.0);

    let center = grid.get_map_center_position();

    for (_, team, skin) in config_game.legends.iter() {
        let agent = spawn_skin_entity(
            &mut commands,
            &mut res_animation_graph,
            &asset_server,
            Transform::from_translation(center + vec3(-100.0, 0.0, 100.0)),
            &skin,
        );

        spawn_fiora(&mut commands, agent);

        commands.entity(agent).insert((
            team.clone(),
            Controller::default(),
            Focus,
            Pickable::IGNORE,
        ));
    }
}

fn spawn_target(
    mut commands: Commands,
    q_t: Query<&AttackTarget>,
    mut res_animation_graph: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    res_navigation_grid: Res<ConfigNavigationGrid>,
    config_game: Res<ConfigGame>,
) {
    if q_t.single().is_ok() {
        return;
    }

    for (_, _, skin) in config_game.legends.iter() {
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
}
