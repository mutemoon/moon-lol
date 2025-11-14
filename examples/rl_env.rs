use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use rand::Rng;

use moon_lol::{
    Action, PluginAnimation, PluginBarrack, PluginCamera, PluginController, PluginCore, PluginMap,
    PluginParticle, PluginSkill, PluginUI,
};

#[derive(Default)]
struct PluginGymEnv;

impl Plugin for PluginGymEnv {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, drive_random_agent);
    }
}

#[derive(Component)]
struct AttackTarget;

#[derive(Component)]
struct RandomAgent {
    timer: Timer,
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

        commands.trigger(moon_lol::CommandAction { entity, action });
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
                resolution: (300, 300).into(),
                position: WindowPosition::At((0, 1000).into()),
                ..default()
            }),
            ..default()
        }));
        app.add_plugins(PluginCore.build().disable::<PluginBarrack>());
    } else {
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(
            PluginCore
                .build()
                .disable::<PluginMap>()
                .disable::<PluginCamera>()
                .disable::<PluginUI>()
                .disable::<PluginParticle>()
                .disable::<PluginAnimation>()
                .disable::<PluginController>()
                .disable::<PluginSkill>(),
        );
    }

    app.add_plugins(PluginGymEnv);
    app.update();
}
