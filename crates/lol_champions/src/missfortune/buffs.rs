use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 女枪被动 - 轻挑
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MissFortuneLoveTap" })]
pub struct BuffMissFortuneLoveTap {
    pub bonus_damage: f32,
    pub timer: Timer,
}

impl BuffMissFortuneLoveTap {
    pub fn new(bonus_damage: f32, duration: f32) -> Self {
        Self {
            bonus_damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 女枪W - 大步流星（移速和攻速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MissFortuneW" })]
pub struct BuffMissFortuneW {
    pub movespeed_bonus: f32,
    pub attackspeed_bonus: f32,
    pub timer: Timer,
}

impl BuffMissFortuneW {
    pub fn new(movespeed_bonus: f32, attackspeed_bonus: f32, duration: f32) -> Self {
        Self {
            movespeed_bonus,
            attackspeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 女枪E - 弹射（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MissFortuneE" })]
pub struct BuffMissFortuneE {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffMissFortuneE {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
