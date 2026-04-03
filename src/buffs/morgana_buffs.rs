use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 莫甘娜被动 - 灵魂虹吸（法术吸血）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MorganaPassive" })]
pub struct BuffMorganaPassive {
    pub lifesteal_percent: f32,
    pub timer: Timer,
}

impl BuffMorganaPassive {
    pub fn new(lifesteal_percent: f32, duration: f32) -> Self {
        Self {
            lifesteal_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 莫甘娜Q - 暗影禁锢（禁锢）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MorganaQ" })]
pub struct BuffMorganaQ {
    pub root_duration: f32,
    pub timer: Timer,
}

impl BuffMorganaQ {
    pub fn new(root_duration: f32, duration: f32) -> Self {
        Self {
            root_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 莫甘娜E - 黑暗护盾（免疫控制）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MorganaE" })]
pub struct BuffMorganaE {
    pub shield_amount: f32,
    pub immune_cc: bool,
    pub timer: Timer,
}

impl BuffMorganaE {
    pub fn new(shield_amount: f32, immune_cc: bool, duration: f32) -> Self {
        Self {
            shield_amount,
            immune_cc,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 莫甘娜R - 灵魂枷锁（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MorganaR" })]
pub struct BuffMorganaR {
    pub damage: f32,
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffMorganaR {
    pub fn new(damage: f32, stun_duration: f32, duration: f32) -> Self {
        Self {
            damage,
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
