use bevy::prelude::*;
use crate::Buff;

/// 奈德丽被动 - 草丛掠食（移速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NidaleePassive" })]
pub struct BuffNidaleePassive {
    pub movespeed_bonus: f32,
    pub timer: Timer,
}

impl BuffNidaleePassive {
    pub fn new(movespeed_bonus: f32, duration: f32) -> Self {
        Self {
            movespeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 奈德丽Q - 投掷标枪（标记）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NidaleeQ" })]
pub struct BuffNidaleeQ {
    pub damage: f32,
    pub timer: Timer,
}

impl BuffNidaleeQ {
    pub fn new(damage: f32, duration: f32) -> Self {
        Self {
            damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 奈德丽W - 丛林伏击（陷阱减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NidaleeW" })]
pub struct BuffNidaleeW {
    pub damage: f32,
    pub timer: Timer,
}

impl BuffNidaleeW {
    pub fn new(damage: f32, duration: f32) -> Self {
        Self {
            damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 奈德丽E - 野性激发（治疗和攻速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NidaleeE" })]
pub struct BuffNidaleeE {
    pub heal_amount: f32,
    pub attackspeed_bonus: f32,
    pub timer: Timer,
}

impl BuffNidaleeE {
    pub fn new(heal_amount: f32, attackspeed_bonus: f32, duration: f32) -> Self {
        Self {
            heal_amount,
            attackspeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
