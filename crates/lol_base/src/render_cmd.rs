use bevy::prelude::*;

#[derive(EntityEvent)]
pub struct CommandSkinParticleSpawn {
    pub entity: Entity,
    pub hash: u32,
}

#[derive(EntityEvent)]
pub struct CommandSkinParticleDespawn {
    pub entity: Entity,
    pub hash: u32,
}

#[derive(EntityEvent)]
pub struct CommandAnimationPlay {
    pub entity: Entity,
    pub hash: u32,
    pub repeat: bool,
    pub duration: Option<f32>,
}
