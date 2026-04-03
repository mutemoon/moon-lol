use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 诺提勒斯被动 - 猛冲重击（击飞）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NautilusPassive" })]
pub struct BuffNautilusPassive {
    pub damage: f32,
    pub knockup_duration: f32,
    pub timer: Timer,
}

impl BuffNautilusPassive {
    pub fn new(damage: f32, knockup_duration: f32, duration: f32) -> Self {
        Self {
            damage,
            knockup_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 诺提勒斯W - 泰坦的愤怒（护盾）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NautilusW" })]
pub struct BuffNautilusW {
    pub shield_amount: f32,
    pub timer: Timer,
}

impl BuffNautilusW {
    pub fn new(shield_amount: f32, duration: f32) -> Self {
        Self {
            shield_amount,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 诺提勒斯E - 潮汐（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NautilusE" })]
pub struct BuffNautilusE {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffNautilusE {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
