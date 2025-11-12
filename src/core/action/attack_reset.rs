use bevy::prelude::*;
use bevy_behave::prelude::BehaveTrigger;

use crate::CommandAttackReset;

#[derive(Debug, Clone)]
pub struct ActionAttackReset;

pub fn on_action_attack_reset(
    trigger: Trigger<BehaveTrigger<ActionAttackReset>>,
    mut commands: Commands,
) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let _event = trigger.inner();

    commands.entity(entity).trigger(CommandAttackReset);

    commands.trigger(ctx.success());
}
