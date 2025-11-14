use bevy::prelude::*;
use bevy_behave::prelude::BehaveTrigger;

use crate::CommandAttackReset;

#[derive(Debug, Clone)]
pub struct ActionAttackReset;

pub fn on_action_attack_reset(
    trigger: On<BehaveTrigger<ActionAttackReset>>,
    mut commands: Commands,
) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let _event = trigger.inner();

    commands.trigger(CommandAttackReset { entity });

    commands.trigger(ctx.success());
}
