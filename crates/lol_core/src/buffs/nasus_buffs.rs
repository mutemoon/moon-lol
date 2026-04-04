use bevy::prelude::*;

use crate::base::buff::Buff;

/// 沙漠死神被动 - 噬魂者（吸血）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NasusPassive" })]
pub struct BuffNasusPassive {
    pub lifesteal_percent: f32,
    pub timer: Timer,
}

impl BuffNasusPassive {
    pub fn new(lifesteal_percent: f32, duration: f32) -> Self {
        Self {
            lifesteal_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 沙漠死神W - 枯萎（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NasusW" })]
pub struct BuffNasusW {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffNasusW {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 沙漠死神E - 灵魂烈火（减甲）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NasusE" })]
pub struct BuffNasusE {
    pub armor_reduction: f32,
    pub damage: f32,
    pub timer: Timer,
}

impl BuffNasusE {
    pub fn new(armor_reduction: f32, damage: f32, duration: f32) -> Self {
        Self {
            armor_reduction,
            damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 沙漠死神R - 死神降临（增厚和双抗）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NasusR" })]
pub struct BuffNasusR {
    pub bonus_health: f32,
    pub armor_bonus: f32,
    pub magic_resist_bonus: f32,
    pub timer: Timer,
}

impl BuffNasusR {
    pub fn new(
        bonus_health: f32,
        armor_bonus: f32,
        magic_resist_bonus: f32,
        duration: f32,
    ) -> Self {
        Self {
            bonus_health,
            armor_bonus,
            magic_resist_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
