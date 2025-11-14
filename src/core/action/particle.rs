use bevy::prelude::*;
use bevy_behave::prelude::BehaveTrigger;

use crate::{CommandParticleDespawn, CommandParticleSpawn};

#[derive(Debug, Clone)]
pub struct ActionParticleSpawn {
    pub hash: u32,
}

#[derive(Debug, Clone)]
pub struct ActionParticleDespawn {
    pub hash: u32,
}

pub fn on_action_particle_spawn(
    trigger: On<BehaveTrigger<ActionParticleSpawn>>,
    mut commands: Commands,
) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let event = trigger.inner();

    commands.trigger(CommandParticleSpawn {
        entity,
        particle: event.hash,
    });
    commands.trigger(ctx.success());
}

pub fn on_action_particle_despawn(
    trigger: On<BehaveTrigger<ActionParticleDespawn>>,
    mut commands: Commands,
) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let event = trigger.inner();

    commands.trigger(CommandParticleDespawn { entity, hash: event.hash });
    commands.trigger(ctx.success());
}
