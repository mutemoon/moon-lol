use bevy::prelude::*;

use crate::base::buff::Buff;

/// 冰晶凤凰被动 - 冰霜奴役
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LissandraPassive" })]
pub struct BuffLissandraPassive {
    pub damage: f32,
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffLissandraPassive {
    pub fn new(damage: f32, slow_percent: f32, duration: f32) -> Self {
        Self {
            damage,
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 冰晶凤凰Q - 碎冰减速
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LissandraQ" })]
pub struct BuffLissandraQ {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffLissandraQ {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 冰晶凤凰W - 冰霜之环禁锢
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LissandraW" })]
pub struct BuffLissandraW {
    pub root_duration: f32,
    pub timer: Timer,
}

impl BuffLissandraW {
    pub fn new(root_duration: f32, duration: f32) -> Self {
        Self {
            root_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 冰晶凤凰R - 冰封陵墓（冰箱）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LissandraR" })]
pub struct BuffLissandraR {
    pub invulnerable: bool,
    pub heal_amount: f32,
    pub timer: Timer,
}

impl BuffLissandraR {
    pub fn new(invulnerable: bool, heal_amount: f32, duration: f32) -> Self {
        Self {
            invulnerable,
            heal_amount,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
