use bevy::prelude::*;
use crate::Buff;

/// 凯南被动 - 风暴印记（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KennenMarkOfStorm" })]
pub struct BuffKennenMarkOfStorm {
    pub stacks: u8,
    pub timer: Timer,
}

impl BuffKennenMarkOfStorm {
    pub fn new(stacks: u8, duration: f32) -> Self {
        Self {
            stacks,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn add_stacks(&mut self, amount: u8) {
        self.stacks = (self.stacks + amount).min(3);
    }
}

/// 凯南E - 闪电冲刺（移速和免疫）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KennenE" })]
pub struct BuffKennenE {
    pub movespeed_bonus: f32,
    pub attackspeed_bonus: f32,
    pub immune: bool,
    pub timer: Timer,
}

impl BuffKennenE {
    pub fn new(movespeed_bonus: f32, attackspeed_bonus: f32, duration: f32) -> Self {
        Self {
            movespeed_bonus,
            attackspeed_bonus,
            immune: true,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 凯南R - 风暴龙卷风（双抗加成）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KennenR" })]
pub struct BuffKennenR {
    pub armor_bonus: f32,
    pub magic_resist_bonus: f32,
    pub timer: Timer,
}

impl BuffKennenR {
    pub fn new(armor_bonus: f32, magic_resist_bonus: f32, duration: f32) -> Self {
        Self {
            armor_bonus,
            magic_resist_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
