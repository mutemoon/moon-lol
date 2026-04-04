use bevy::prelude::*;

use crate::base::buff::Buff;

/// 希维尔W - 弹射（攻速加成）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SivirW" })]
pub struct BuffSivirW {
    pub attackspeed_bonus: f32,
    pub timer: Timer,
}

impl BuffSivirW {
    pub fn new(attackspeed_bonus: f32, duration: f32) -> Self {
        Self {
            attackspeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
