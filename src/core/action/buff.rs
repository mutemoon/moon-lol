use std::sync::Arc;

use bevy::prelude::*;
use bevy_behave::prelude::BehaveTrigger;

type BundleSpawner = Arc<dyn Fn(&mut EntityCommands) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct ActionBuffSpawn {
    pub bundle: BundleSpawner,
}

pub fn on_action_buff_spawn(
    trigger: Trigger<BehaveTrigger<ActionBuffSpawn>>,
    mut commands: Commands,
) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let event = trigger.inner();

    (event.bundle)(&mut commands.entity(entity));

    commands.trigger(ctx.success());
}
