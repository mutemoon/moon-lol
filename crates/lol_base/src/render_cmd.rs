use bevy::prelude::*;

#[derive(EntityEvent)]
pub struct CommandAnimationPlay {
    pub entity: Entity,
    pub hash: String,
    pub repeat: bool,
    pub duration: Option<f32>,
}
