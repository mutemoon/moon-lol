use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 奥莉安娜被动 - 发条上弦（额外伤害）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "OriannaPassive" })]
pub struct BuffOriannaPassive {
    pub bonus_damage: f32,
    pub timer: Timer,
}

impl BuffOriannaPassive {
    pub fn new(bonus_damage: f32, duration: f32) -> Self {
        Self {
            bonus_damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 奥莉安娜W - 命令：失谐（加速/减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "OriannaW" })]
pub struct BuffOriannaW {
    pub movespeed_bonus: f32,
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffOriannaW {
    pub fn new(movespeed_bonus: f32, slow_percent: f32, duration: f32) -> Self {
        Self {
            movespeed_bonus,
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 奥莉安娜E - 命令：保护（护盾）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "OriannaE" })]
pub struct BuffOriannaE {
    pub shield_amount: f32,
    pub armor_bonus: f32,
    pub timer: Timer,
}

impl BuffOriannaE {
    pub fn new(shield_amount: f32, armor_bonus: f32, duration: f32) -> Self {
        Self {
            shield_amount,
            armor_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
