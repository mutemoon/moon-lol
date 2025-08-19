use bevy::prelude::*;

use crate::core::{CommandAttackCancel, CommandAttackCast, CommandNavigationTo, CommandTargetSet};

#[derive(Event)]
pub struct CommandCommandAttack {
    pub target: Entity,
}

#[derive(Event)]
pub struct CommandCommandMoveTo(pub Vec2);

pub struct PluginCommand;

impl Plugin for PluginCommand {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandCommandAttack>();
        app.add_event::<CommandCommandMoveTo>();
        app.add_observer(command_attack);
    }
}

pub fn command_attack(trigger: Trigger<CommandCommandAttack>, mut commands: Commands) {
    commands.trigger_targets(
        CommandTargetSet {
            target: trigger.target,
        },
        trigger.target(),
    );
    commands.trigger_targets(CommandAttackCast, trigger.target());
}

pub fn command_move_to(trigger: Trigger<CommandCommandMoveTo>, mut commands: Commands) {
    commands.trigger_targets(CommandAttackCancel, trigger.target());
    commands.trigger_targets(CommandNavigationTo(trigger.0), trigger.target());
}
