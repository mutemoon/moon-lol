use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 斯莫德W - 深火烙印（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SmolderW" })]
pub struct BuffSmolderW {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffSmolderW {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
