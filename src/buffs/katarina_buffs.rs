use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 卡特琳娜被动 - 贪婪（参与击杀减少冷却）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KatarinaVoracity" })]
pub struct BuffKatarinaVoracity {
    pub cooldown_reduction: f32,
    pub timer: Timer,
}

impl BuffKatarinaVoracity {
    pub fn new(cooldown_reduction: f32, duration: f32) -> Self {
        Self {
            cooldown_reduction,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 卡特琳娜W - 准备（移速加成）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KatarinaW" })]
pub struct BuffKatarinaW {
    pub movespeed_bonus: f32,
    pub timer: Timer,
}

impl BuffKatarinaW {
    pub fn new(movespeed_bonus: f32, duration: f32) -> Self {
        Self {
            movespeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
