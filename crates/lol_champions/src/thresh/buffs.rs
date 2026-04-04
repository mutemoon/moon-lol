use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 锤石Q - 死亡判决（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "ThreshQ" })]
pub struct BuffThreshQ {
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffThreshQ {
    pub fn new(stun_duration: f32, duration: f32) -> Self {
        Self {
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 锤石E - 厄运之牢（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "ThreshE" })]
pub struct BuffThreshE {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffThreshE {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
