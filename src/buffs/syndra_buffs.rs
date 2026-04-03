use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 辛德拉E - 驱使法球（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SyndraE" })]
pub struct BuffSyndraE {
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffSyndraE {
    pub fn new(stun_duration: f32, duration: f32) -> Self {
        Self {
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
