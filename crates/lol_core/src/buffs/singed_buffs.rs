use bevy::prelude::*;

use crate::base::buff::Buff;

/// 辛吉德E - 致命搅拌（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SingedE" })]
pub struct BuffSingedE {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffSingedE {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
