use bevy::prelude::*;

use crate::base::buff::Buff;

/// 雷恩加尔E - 套索（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RengarE" })]
pub struct BuffRengarE {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffRengarE {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 雷恩加尔R - 狩猎本能（移速加成）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RengarR" })]
pub struct BuffRengarR {
    pub movespeed_bonus: f32,
    pub timer: Timer,
}

impl BuffRengarR {
    pub fn new(movespeed_bonus: f32, duration: f32) -> Self {
        Self {
            movespeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
