use bevy::asset::AssetPlugin;
use bevy::prelude::*;

use lol_config::{ConfigGame, ConfigNavigationGrid};
use lol_core::Team;

use moon_lol::abilities::PluginAbilities;
use moon_lol::core::{spawn_skin_entity, Action, Controller, Focus, Health, PluginCore};
use moon_lol::entities::{spawn_fiora, PluginBarrack, PluginEntities};
use rand::Rng;

#[derive(Default)]
struct PluginGymEnv;

impl Plugin for PluginGymEnv {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_fiora_player);
        app.add_systems(FixedUpdate, spawn_target);
        app.add_systems(FixedUpdate, drive_random_agent);
    }
}

#[derive(Component)]
struct AttackTarget;

#[derive(Component)]
struct RandomAgent {
    timer: Timer,
}

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

        commands.entity(agent).insert(RandomAgent {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        });
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

fn drive_random_agent(
    mut commands: Commands,
    mut agents: Query<(Entity, &mut RandomAgent, &Transform)>,
    q_target: Query<Entity, With<AttackTarget>>,
    time: Res<Time<Fixed>>,
) {
    for (entity, mut agent, transform) in agents.iter_mut() {
        agent.timer.tick(time.delta());
        if !agent.timer.just_finished() {
            continue;
        }

        let mut rng = rand::rng();
        let choice = rng.random_range(0..3);

        let action = match choice {
            0 => Action::Attack(q_target.single().unwrap()),

            1 => {
                let angle = rng.random_range(0.0f32..std::f32::consts::TAU);
                let radius = rng.random_range(50.0f32..200.0f32);
                let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                Action::Move(transform.translation.xz() + offset)
            }

            2 => Action::Stop,

            _ => Action::Stop,
        };

        commands
            .entity(entity)
            .trigger(moon_lol::core::CommandAction { action });
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let render_flag = args.iter().any(|a| a == "--render")
        || std::env::var("MOON_RL_RENDER")
            .map(|v| {
                let v = v.to_ascii_lowercase();
                v == "1" || v == "true" || v == "yes"
            })
            .unwrap_or(true);

    let mut app = App::new();

    if render_flag {
        app.add_plugins(DefaultPlugins.build().set(WindowPlugin {
            primary_window: Some(Window {
                title: "classic 1v1 fiora".to_string(),
                resolution: (300.0, 300.0).into(),
                position: WindowPosition::At((0, 1000).into()),
                ..default()
            }),
            ..default()
        }));
        app.add_plugins(PluginCore);
        app.add_plugins(PluginEntities.build().disable::<PluginBarrack>());
        app.add_plugins(PluginAbilities);
    } else {
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(
            PluginCore
                .build()
                .disable::<moon_lol::core::PluginMap>()
                .disable::<moon_lol::core::PluginCamera>()
                .disable::<moon_lol::core::PluginUI>()
                .disable::<moon_lol::core::PluginParticle>()
                .disable::<moon_lol::core::PluginAnimation>()
                .disable::<moon_lol::core::PluginController>()
                .disable::<moon_lol::core::PluginSkill>(),
        );
    }

    app.add_plugins(PluginGymEnv);
    app.update();
}
