use bevy::prelude::*;

use crate::base::buff::Buff;

/// 芮尔W - 挥击（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RellW" })]
pub struct BuffRellW {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffRellW {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 芮尔E - 引爆（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RellE" })]
pub struct BuffRellE {
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffRellE {
    pub fn new(stun_duration: f32, duration: f32) -> Self {
        Self {
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 芮尔R - 极灵涤荡（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RellR" })]
pub struct BuffRellR {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffRellR {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
