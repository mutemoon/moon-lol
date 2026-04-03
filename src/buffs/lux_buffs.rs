use bevy::prelude::*;
use crate::Buff;

/// 拉克丝被动 - 照明标记
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LuxIllumination" })]
pub struct BuffLuxIllumination {
    pub bonus_damage: f32,
    pub timer: Timer,
}

impl BuffLuxIllumination {
    pub fn new(bonus_damage: f32, duration: f32) -> Self {
        Self {
            bonus_damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 拉克丝Q - 光之束缚（禁锢）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LuxQ" })]
pub struct BuffLuxQ {
    pub root_duration: f32,
    pub timer: Timer,
}

impl BuffLuxQ {
    pub fn new(root_duration: f32, duration: f32) -> Self {
        Self {
            root_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 拉克丝W - 曲光屏障（护盾）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LuxW" })]
pub struct BuffLuxW {
    pub shield_amount: f32,
    pub timer: Timer,
}

impl BuffLuxW {
    pub fn new(shield_amount: f32, duration: f32) -> Self {
        Self {
            shield_amount,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 拉克丝E - 透光奇点（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LuxE" })]
pub struct BuffLuxE {
    pub slow_percent: f32,
    pub damage: f32,
    pub timer: Timer,
}

impl BuffLuxE {
    pub fn new(slow_percent: f32, damage: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
