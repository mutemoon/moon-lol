use bevy::prelude::*;
use crate::Buff;

/// 卡尔莎被动 - 聚集之火（减少R冷却）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KarmaGatheringFire" })]
pub struct BuffKarmaGatheringFire {
    pub cooldown_reduction: f32,
    pub timer: Timer,
}

impl BuffKarmaGatheringFire {
    pub fn new(duration: f32) -> Self {
        Self {
            cooldown_reduction: 4.0,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 卡尔莎Q - 内心之火减速
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KarmaQ" })]
pub struct BuffKarmaQ {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffKarmaQ {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 卡尔莎W - 专注禁锢
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KarmaW" })]
pub struct BuffKarmaW {
    pub timer: Timer,
}

impl BuffKarmaW {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 卡尔莎E - 鼓舞护盾和移速
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KarmaE" })]
pub struct BuffKarmaE {
    pub shield_amount: f32,
    pub movespeed_bonus: f32,
    pub timer: Timer,
}

impl BuffKarmaE {
    pub fn new(shield_amount: f32, movespeed_bonus: f32, duration: f32) -> Self {
        Self {
            shield_amount,
            movespeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 卡尔莎R强化状态
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KarmaMantra" })]
pub struct BuffKarmaMantra {
    pub enhanced_skill: String,
    pub timer: Timer,
}

impl BuffKarmaMantra {
    pub fn new(enhanced_skill: &str, duration: f32) -> Self {
        Self {
            enhanced_skill: enhanced_skill.to_string(),
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
