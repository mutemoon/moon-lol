use bevy::prelude::*;
use crate::Buff;

/// 卡莎被动 - 等离子标记
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KaisaPlasma" })]
pub struct BuffKaisaPlasma {
    pub stacks: u8,
    pub timer: Timer,
}

impl BuffKaisaPlasma {
    pub fn new(stacks: u8, duration: f32) -> Self {
        Self {
            stacks,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn add_stacks(&mut self, amount: u8) {
        self.stacks = (self.stacks + amount).min(5);
    }
}

/// 卡莎E - 玛西亚的复仇（攻速加成）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KaisaE" })]
pub struct BuffKaisaE {
    pub attackspeed_bonus: f32,
    pub timer: Timer,
}

impl BuffKaisaE {
    pub fn new(attackspeed_bonus: f32, duration: f32) -> Self {
        Self {
            attackspeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 卡莎R - 杀手本能（护盾）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KaisaR" })]
pub struct BuffKaisaR {
    pub shield_amount: f32,
    pub timer: Timer,
}

impl BuffKaisaR {
    pub fn new(shield_amount: f32, duration: f32) -> Self {
        Self {
            shield_amount,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
