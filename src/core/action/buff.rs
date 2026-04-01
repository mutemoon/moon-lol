use std::sync::Arc;

use bevy::prelude::*;

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
