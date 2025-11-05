mod animation;
mod attack_reset;
mod buff;
mod command;
mod damage;
mod dash;
mod particle;

pub use animation::*;
pub use attack_reset::*;
pub use buff::*;
pub use command::*;
pub use damage::*;
pub use dash::*;
pub use particle::*;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::{
    Attack, CommandAttackAutoStart, CommandAttackAutoStop, CommandMovement, CommandRunStart,
    CommandSkillStart, MovementAction, RunTarget,
};

#[derive(Default)]
pub struct PluginAction;

impl Plugin for PluginAction {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandAction>();

        app.add_observer(on_action_animation_play);
        app.add_observer(on_action_attack_reset);
        app.add_observer(on_action_buff_spawn);
        app.add_observer(on_action_dash_end);
        app.add_observer(on_action_dash);
        app.add_observer(on_action_particle_despawn);
        app.add_observer(on_action_particle_spawn);
        app.add_observer(on_action_command);
        app.add_observer(on_attack_damage);

        app.add_observer(on_command_action);
    }
}

#[derive(Event)]
pub struct CommandAction {
    pub action: Action,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    Attack(Entity),
    Move(Vec2),
    Stop,
    Skill { index: usize, point: Vec3 },
}

fn on_command_action(
    trigger: Trigger<CommandAction>,
    mut commands: Commands,
    q_attack: Query<&Attack>,
) {
    let entity = trigger.target();

    match trigger.event().action {
        Action::Attack(target) => {
            let attack = q_attack.get(entity).unwrap();
            commands
                .entity(entity)
                .trigger(CommandAttackAutoStart { target });
        }
        Action::Move(target) => {
            commands.entity(entity).trigger(CommandAttackAutoStop);
            commands.entity(entity).trigger(CommandRunStart {
                target: RunTarget::Position(target),
            });
            // commands.entity(entity).trigger(CommandAnimationPlay {
            //     hash: hash_bin("Run"),
            //     repeat: true,
            //     ..default()
            // });
        }
        Action::Skill { index, point } => {
            commands.trigger_targets(CommandSkillStart { index, point }, trigger.target());
        }
        Action::Stop => {
            commands.trigger_targets(CommandAttackAutoStop, trigger.target());
            commands.trigger_targets(
                CommandMovement {
                    priority: 0,
                    action: MovementAction::Stop,
                },
                trigger.target(),
            );
        }
    }
}
