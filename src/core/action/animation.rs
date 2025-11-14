use bevy::prelude::*;
use bevy_behave::prelude::BehaveTrigger;

use crate::CommandAnimationPlay;

#[derive(Debug, Clone)]
pub struct ActionAnimationPlay {
    pub hash: u32,
}

pub fn on_action_animation_play(
    trigger: On<BehaveTrigger<ActionAnimationPlay>>,
    mut commands: Commands,
) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let event = trigger.inner();

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: event.hash,
        repeat: false,
        duration: None,
    });
    commands.trigger(ctx.success());
}
