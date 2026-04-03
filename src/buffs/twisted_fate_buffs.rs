use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 崔斯特W - 选牌-金牌（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "TwistedFateWStun" })]
pub struct BuffTwistedFateWStun {
    pub timer: Timer,
}

impl BuffTwistedFateWStun {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 崔斯特W - 选牌-红牌（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "TwistedFateWSlow" })]
pub struct BuffTwistedFateWSlow {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffTwistedFateWSlow {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
