pub mod damage;
pub mod dash;
pub mod delayed_damage;
pub mod displace;
pub mod knockback;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::action::damage::on_action_damage;
use crate::action::dash::{
    on_action_dash, on_dash_end, on_dash_start_attach_damage, on_movement_block_add_cancel_dash,
    update_dash_damage, update_tracking_dash,
};
use crate::action::delayed_damage::{on_action_delayed_damage, update_delayed_damage};
use crate::action::displace::{on_action_displace, update_grabbed_entities};
use crate::action::knockback::on_command_knockback;
use crate::attack_auto::{CommandAttackAutoStart, CommandAttackAutoStop};
use crate::movement::{CommandMovement, MovementAction};
use crate::run::{CommandRunStart, RunTarget};
use crate::skill::{CommandSkillBeforeStart, CommandSkillLevelUp, CommandSkillStart};

#[derive(Default)]
pub struct PluginAction;

impl Plugin for PluginAction {
    fn build(&self, app: &mut App) {
        app.add_observer(on_action_dash);
        app.add_observer(on_dash_end);
        app.add_observer(on_movement_block_add_cancel_dash);
        app.add_observer(on_dash_start_attach_damage);
        app.add_observer(on_action_damage);
        app.add_observer(on_action_delayed_damage);
        app.add_observer(on_action_displace);
        app.add_observer(on_command_knockback);

        app.add_observer(on_command_action);

        app.add_systems(FixedUpdate, update_dash_damage);
        app.add_systems(FixedUpdate, update_tracking_dash);
        app.add_systems(FixedUpdate, update_delayed_damage);
        app.add_systems(FixedUpdate, update_grabbed_entities);
    }
}

#[derive(EntityEvent)]
pub struct CommandAction {
    pub entity: Entity,
    pub action: Action,
}

#[derive(Clone, Serialize, Deserialize, Reflect, Debug)]
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
