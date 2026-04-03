use bevy::prelude::*;
use crate::Buff;

/// 璐璐被动 - Pix（小精灵）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LuluPassive" })]
pub struct BuffLuluPassive {
    pub damage: f32,
    pub timer: Timer,
}

impl BuffLuluPassive {
    pub fn new(damage: f32, duration: f32) -> Self {
        Self {
            damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 璐璐W - 奇思妙想（变形或加速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LuluW" })]
pub struct BuffLuluW {
    pub polymorph: bool,
    pub attackspeed_bonus: f32,
    pub movespeed_bonus: f32,
    pub timer: Timer,
}

impl BuffLuluW {
    pub fn new(polymorph: bool, attackspeed_bonus: f32, movespeed_bonus: f32, duration: f32) -> Self {
        Self {
            polymorph,
            attackspeed_bonus,
            movespeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 璐璐E - 帮助皮克斯（护盾）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LuluE" })]
pub struct BuffLuluE {
    pub shield_amount: f32,
    pub timer: Timer,
}

impl BuffLuluE {
    pub fn new(shield_amount: f32, duration: f32) -> Self {
        Self {
            shield_amount,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 璐璐R - 野性生长（击飞和增厚）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LuluR" })]
pub struct BuffLuluR {
    pub bonus_health: f32,
    pub knockup: bool,
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffLuluR {
    pub fn new(bonus_health: f32, knockup: bool, slow_percent: f32, duration: f32) -> Self {
        Self {
            bonus_health,
            knockup,
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
