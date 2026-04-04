use bevy::prelude::*;

use crate::base::buff::Buff;

/// 斯维因W - 帝国钩索（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SwainW" })]
pub struct BuffSwainW {
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffSwainW {
    pub fn new(stun_duration: f32, duration: f32) -> Self {
        Self {
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
