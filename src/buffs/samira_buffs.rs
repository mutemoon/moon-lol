use bevy::prelude::*;
use crate::Buff;

/// 莎弥拉E - 螺旋利刃（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SamiraE" })]
pub struct BuffSamiraE {
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffSamiraE {
    pub fn new(stun_duration: f32, duration: f32) -> Self {
        Self {
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
