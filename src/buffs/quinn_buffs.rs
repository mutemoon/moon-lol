use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 奎因W - 高度感知（攻速+移速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "QuinnW" })]
pub struct BuffQuinnW {
    pub attackspeed_bonus: f32,
    pub movespeed_bonus: f32,
    pub timer: Timer,
}

impl BuffQuinnW {
    pub fn new(attackspeed_bonus: f32, movespeed_bonus: f32, duration: f32) -> Self {
        Self {
            attackspeed_bonus,
            movespeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 奎因E -  vaults（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "QuinnE" })]
pub struct BuffQuinnE {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffQuinnE {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
