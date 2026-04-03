use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 瑞兹W - 符文禁锢（禁锢）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RyzeW" })]
pub struct BuffRyzeW {
    pub root_duration: f32,
    pub timer: Timer,
}

impl BuffRyzeW {
    pub fn new(root_duration: f32, duration: f32) -> Self {
        Self {
            root_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 瑞兹E - 符能迸发（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RyzeE" })]
pub struct BuffRyzeE {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffRyzeE {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
