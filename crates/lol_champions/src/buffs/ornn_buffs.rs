use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 奥恩Q - 火山裂缝（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "OrnnQ" })]
pub struct BuffOrnnQ {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffOrnnQ {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 奥恩W - 吹息（脆弱效果，受到额外伤害）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "OrnnW" })]
pub struct BuffOrnnW {
    pub brittle_percent: f32,
    pub timer: Timer,
}

impl BuffOrnnW {
    pub fn new(brittle_percent: f32, duration: f32) -> Self {
        Self {
            brittle_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 奥恩R - 熔铸之神呼唤（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "OrnnR" })]
pub struct BuffOrnnR {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffOrnnR {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
