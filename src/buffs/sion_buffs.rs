use bevy::prelude::*;
use crate::Buff;

/// 赛恩Q - 残虐猛击（击飞）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SionQ" })]
pub struct BuffSionQ {
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffSionQ {
    pub fn new(stun_duration: f32, duration: f32) -> Self {
        Self {
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 赛恩E - 枯萎（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SionE" })]
pub struct BuffSionE {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffSionE {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
