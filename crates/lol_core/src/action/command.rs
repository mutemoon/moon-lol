use std::sync::Arc;

use bevy::prelude::*;

type BundleSpawner = Arc<dyn Fn(&mut EntityCommands) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct ActionCommand {
    pub bundle: BundleSpawner,
}
