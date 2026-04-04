use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 易大师被动 - 双重打击
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MasterYiDoubleStrike" })]
pub struct BuffMasterYiDoubleStrike {
    pub attacks_until_trigger: u8,
    pub bonus_damage: f32,
    pub timer: Timer,
}

impl BuffMasterYiDoubleStrike {
    pub fn new(attacks_until_trigger: u8, bonus_damage: f32, duration: f32) -> Self {
        Self {
            attacks_until_trigger,
            bonus_damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 易大师W - 冥想（治疗和减伤）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MasterYiW" })]
pub struct BuffMasterYiW {
    pub heal_amount: f32,
    pub damage_reduction: f32,
    pub timer: Timer,
}

impl BuffMasterYiW {
    pub fn new(heal_amount: f32, damage_reduction: f32, duration: f32) -> Self {
        Self {
            heal_amount,
            damage_reduction,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 易大师E - 无双重伤（额外真实伤害）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MasterYiE" })]
pub struct BuffMasterYiE {
    pub bonus_damage_percent: f32,
    pub timer: Timer,
}

impl BuffMasterYiE {
    pub fn new(bonus_damage_percent: f32, duration: f32) -> Self {
        Self {
            bonus_damage_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 易大师R - 高原血统（攻速和移速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MasterYiR" })]
pub struct BuffMasterYiR {
    pub attackspeed_bonus: f32,
    pub movespeed_bonus: f32,
    pub timer: Timer,
}

impl BuffMasterYiR {
    pub fn new(attackspeed_bonus: f32, movespeed_bonus: f32, duration: f32) -> Self {
        Self {
            attackspeed_bonus,
            movespeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
