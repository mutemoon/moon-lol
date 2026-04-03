use bevy::prelude::*;
use crate::Buff;

/// 瑟庄妮Q - 极冰冲击（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SejuaniQ" })]
pub struct BuffSejuaniQ {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffSejuaniQ {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 瑟庄妮W - 冰霜护甲（护盾）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SejuaniW" })]
pub struct BuffSejuaniW {
    pub shield_amount: f32,
    pub timer: Timer,
}

impl BuffSejuaniW {
    pub fn new(shield_amount: f32, duration: f32) -> Self {
        Self {
            shield_amount,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 瑟庄妮E - 永冻领域（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SejuaniE" })]
pub struct BuffSejuaniE {
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffSejuaniE {
    pub fn new(stun_duration: f32, duration: f32) -> Self {
        Self {
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
