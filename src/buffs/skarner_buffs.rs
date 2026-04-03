use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 斯卡纳R - 晶锥共鸣（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SkarnerR" })]
pub struct BuffSkarnerR {
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffSkarnerR {
    pub fn new(stun_duration: f32, duration: f32) -> Self {
        Self {
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
