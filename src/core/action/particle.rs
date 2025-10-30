use bevy::prelude::*;
use bevy_behave::prelude::BehaveTrigger;

use crate::core::{CommandParticleDespawn, CommandParticleSpawn};

#[derive(Debug, Clone)]
pub struct ActionParticleSpawn {
    pub hash: u32,
}

#[derive(Debug, Clone)]
pub struct ActionParticleDespawn {
    pub hash: u32,
}

pub fn on_action_particle_spawn(
    trigger: Trigger<BehaveTrigger<ActionParticleSpawn>>,
    mut commands: Commands,
) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let event = trigger.inner();

    commands.entity(entity).trigger(CommandParticleSpawn {
        particle: event.hash,
    });
    commands.trigger(ctx.success());
}

pub fn on_action_particle_despawn(
    trigger: Trigger<BehaveTrigger<ActionParticleDespawn>>,
    mut commands: Commands,
) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let event = trigger.inner();

    commands.entity(entity).trigger(CommandParticleDespawn {
        particle: event.hash,
    });
    commands.trigger(ctx.success());
}
