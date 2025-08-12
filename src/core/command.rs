use bevy::prelude::*;

use crate::core::{CommandAttackCast, CommandTargetSet};

#[derive(Event)]
pub struct CommandCommandAttack {
    pub target: Entity,
}

pub struct PluginCommand;

impl Plugin for PluginCommand {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandCommandAttack>();
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
