use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 蕾欧娜被动 - 阳光标记
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LeonaSunlight" })]
pub struct BuffLeonaSunlight {
    pub damage: f32,
    pub timer: Timer,
}

impl BuffLeonaSunlight {
    pub fn new(damage: f32, duration: f32) -> Self {
        Self {
            damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 蕾欧娜Q - 日蚀（强化普攻眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LeonaQ" })]
pub struct BuffLeonaQ {
    pub bonus_damage: f32,
    pub timer: Timer,
}

impl BuffLeonaQ {
    pub fn new(bonus_damage: f32, duration: f32) -> Self {
        Self {
            bonus_damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 蕾欧娜W - 日炎（伤害减免和双抗）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LeonaW" })]
pub struct BuffLeonaW {
    pub damage_reduction: f32,
    pub armor_bonus: f32,
    pub magic_resist_bonus: f32,
    pub timer: Timer,
}

impl BuffLeonaW {
    pub fn new(
        damage_reduction: f32,
        armor_bonus: f32,
        magic_resist_bonus: f32,
        duration: f32,
    ) -> Self {
        Self {
            damage_reduction,
            armor_bonus,
            magic_resist_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
