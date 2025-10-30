use bevy::prelude::*;
use bevy_behave::prelude::BehaveTrigger;

use crate::core::CommandAnimationPlay;

#[derive(Debug, Clone)]
pub struct ActionAnimationPlay {
    pub hash: u32,
}

pub fn on_action_animation_play(
    trigger: Trigger<BehaveTrigger<ActionAnimationPlay>>,
    mut commands: Commands,
) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let event = trigger.inner();

    commands.entity(entity).trigger(CommandAnimationPlay {
        hash: event.hash,
        repeat: false,
        ..default()
    });
    commands.trigger(ctx.success());
}
