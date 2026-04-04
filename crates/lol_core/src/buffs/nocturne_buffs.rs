use bevy::prelude::*;

use crate::base::buff::Buff;

/// 梦魇被动 - 夜魔翅膀（额外伤害和治疗）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NocturnePassive" })]
pub struct BuffNocturnePassive {
    pub bonus_damage: f32,
    pub heal_amount: f32,
    pub timer: Timer,
}

impl BuffNocturnePassive {
    pub fn new(bonus_damage: f32, heal_amount: f32, duration: f32) -> Self {
        Self {
            bonus_damage,
            heal_amount,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 梦魇Q - 暗影之刃（路径加速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NocturneQ" })]
pub struct BuffNocturneQ {
    pub movespeed_bonus: f32,
    pub ad_bonus: f32,
    pub timer: Timer,
}

impl BuffNocturneQ {
    pub fn new(movespeed_bonus: f32, ad_bonus: f32, duration: f32) -> Self {
        Self {
            movespeed_bonus,
            ad_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 梦魇W - 黑暗庇护（攻速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NocturneW" })]
pub struct BuffNocturneW {
    pub attackspeed_bonus: f32,
    pub timer: Timer,
}

impl BuffNocturneW {
    pub fn new(attackspeed_bonus: f32, duration: f32) -> Self {
        Self {
            attackspeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 梦魇E - 无法言喻的恐惧（恐惧）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NocturneE" })]
pub struct BuffNocturneE {
    pub fear_duration: f32,
    pub timer: Timer,
}

impl BuffNocturneE {
    pub fn new(fear_duration: f32, duration: f32) -> Self {
        Self {
            fear_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
