mod animation;
mod attack_reset;
mod buff;
mod command;
mod damage;
mod dash;
mod particle;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    CommandAttackAutoStart, CommandAttackAutoStop, CommandMovement, CommandRunStart,
    CommandSkillBeforeStart, CommandSkillLevelUp, CommandSkillStart, MovementAction, RunTarget,
};

#[derive(Default)]
pub struct PluginAction;

impl Plugin for PluginAction {
    fn build(&self, app: &mut App) {
        app.add_observer(on_dash_end);
        app.add_observer(on_action_damage);

        app.add_observer(on_command_action);

        app.add_systems(FixedUpdate, update_dash_damage);
    }
}

#[derive(EntityEvent)]
pub struct CommandAction {
    pub entity: Entity,
    pub action: Action,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    Attack(Entity),
    Move(Vec2),
    Stop,
    Skill { index: usize, point: Vec2 },
    SkillLevelUp(usize),
}

fn on_command_action(trigger: On<CommandAction>, mut commands: Commands) {
    let entity = trigger.event_target();

    match trigger.action {
        Action::Attack(target) => {
            commands.trigger(CommandAttackAutoStart { entity, target });
        }
        Action::Move(target) => {
            commands.trigger(CommandAttackAutoStop { entity });
            commands.trigger(CommandRunStart {
                entity,
                target: RunTarget::Position(target),
            });
            // commands.entity(entity).trigger(CommandAnimationPlay {
            //     hash: hash_bin("Run"),
            //     repeat: true,
            //     ..default()
            // });
        }
        Action::Skill { index, point } => {
            commands.trigger(CommandSkillBeforeStart {
                entity,
                index,
                point,
            });
            commands.trigger(CommandSkillStart {
                entity,
                index,
                point,
            });
        }
        Action::SkillLevelUp(index) => {
            commands.trigger(CommandSkillLevelUp { entity, index });
        }
        Action::Stop => {
            commands.trigger(CommandAttackAutoStop { entity });
            commands.trigger(CommandMovement {
                entity,
                priority: 0,
                action: MovementAction::Stop,
            });
        }
    }
}
