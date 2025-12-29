use std::sync::Arc;

use bevy::prelude::*;
use bevy_behave::prelude::BehaveTrigger;

use crate::BuffOf;

type BundleSpawner = Arc<dyn Fn(&mut EntityCommands) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct ActionBuffSpawn {
    pub bundle: BundleSpawner,
}

impl ActionBuffSpawn {
    pub fn new(bundle: impl Bundle + Clone) -> Self {
        Self {
            bundle: Arc::new(move |commands: &mut EntityCommands| {
                commands.with_related::<BuffOf>(bundle.clone());
            }),
        }
    }
}

pub fn on_action_buff_spawn(trigger: On<BehaveTrigger<ActionBuffSpawn>>, mut commands: Commands) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let event = trigger.inner();

    (event.bundle)(&mut commands.entity(entity));

    commands.trigger(ctx.success());
}
