use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 克烈被动 - 战备（骑乘状态）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KledPassive" })]
pub struct BuffKledPassive {
    pub mounted: bool,
    pub courage: f32,
    pub timer: Timer,
}

impl BuffKledPassive {
    pub fn new(mounted: bool, courage: f32, duration: f32) -> Self {
        Self {
            mounted,
            courage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 克烈Q - 飞索（伤害和拉人）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KledQ" })]
pub struct BuffKledQ {
    pub damage: f32,
    pub timer: Timer,
}

impl BuffKledQ {
    pub fn new(damage: f32, duration: f32) -> Self {
        Self {
            damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 克烈W - 狂暴（攻速加成）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KledW" })]
pub struct BuffKledW {
    pub attackspeed_bonus: f32,
    pub timer: Timer,
}

impl BuffKledW {
    pub fn new(attackspeed_bonus: f32, duration: f32) -> Self {
        Self {
            attackspeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 克烈E - 冲刺（位移和加速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KledE" })]
pub struct BuffKledE {
    pub movespeed_bonus: f32,
    pub timer: Timer,
}

impl BuffKledE {
    pub fn new(movespeed_bonus: f32, duration: f32) -> Self {
        Self {
            movespeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 克烈R - 召集！（冲锋）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KledR" })]
pub struct BuffKledR {
    pub movespeed_bonus: f32,
    pub shield_amount: f32,
    pub timer: Timer,
}

impl BuffKledR {
    pub fn new(movespeed_bonus: f32, shield_amount: f32, duration: f32) -> Self {
        Self {
            movespeed_bonus,
            shield_amount,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
