use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 泰隆W - 尝血利刃（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "TalonW" })]
pub struct BuffTalonW {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffTalonW {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
