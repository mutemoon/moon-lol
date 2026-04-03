use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 拉莫斯Q - 动力球（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RammusQ" })]
pub struct BuffRammusQ {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffRammusQ {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 拉莫斯E - 狂乱嘲讽（嘲讽）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RammusE" })]
pub struct BuffRammusE {
    pub taunt_duration: f32,
    pub timer: Timer,
}

impl BuffRammusE {
    pub fn new(taunt_duration: f32, duration: f32) -> Self {
        Self {
            taunt_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 拉莫斯R - 冲天一击（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RammusR" })]
pub struct BuffRammusR {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffRammusR {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
