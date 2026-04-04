use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 娜美被动 - 潮涌（移速加成）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NamiPassive" })]
pub struct BuffNamiPassive {
    pub movespeed_bonus: f32,
    pub timer: Timer,
}

impl BuffNamiPassive {
    pub fn new(movespeed_bonus: f32, duration: f32) -> Self {
        Self {
            movespeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 娜美Q - 泡泡（禁锢）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NamiQ" })]
pub struct BuffNamiQ {
    pub root_duration: f32,
    pub timer: Timer,
}

impl BuffNamiQ {
    pub fn new(root_duration: f32, duration: f32) -> Self {
        Self {
            root_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 娜美E - 守护（强化攻击）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NamiE" })]
pub struct BuffNamiE {
    pub bonus_damage: f32,
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffNamiE {
    pub fn new(bonus_damage: f32, slow_percent: f32, duration: f32) -> Self {
        Self {
            bonus_damage,
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
