use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 洛W - 华丽登场（击飞）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RakanW" })]
pub struct BuffRakanW {
    pub knockup_duration: f32,
    pub timer: Timer,
}

impl BuffRakanW {
    pub fn new(knockup_duration: f32, duration: f32) -> Self {
        Self {
            knockup_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 洛R - 速度之舞（魅惑+减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RakanR" })]
pub struct BuffRakanR {
    pub charm_duration: f32,
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffRakanR {
    pub fn new(charm_duration: f32, slow_percent: f32, duration: f32) -> Self {
        Self {
            charm_duration,
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
