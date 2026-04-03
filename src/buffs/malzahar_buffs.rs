use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 马尔扎哈被动 - 虚空穿越
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MalzaharVoidShift" })]
pub struct BuffMalzaharVoidShift {
    pub damage_reduction: f32,
    pub immune: bool,
    pub timer: Timer,
}

impl BuffMalzaharVoidShift {
    pub fn new(damage_reduction: f32, immune: bool, duration: f32) -> Self {
        Self {
            damage_reduction,
            immune,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 马尔扎哈Q - 虚空呼唤（沉默）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MalzaharQ" })]
pub struct BuffMalzaharQ {
    pub silence_duration: f32,
    pub timer: Timer,
}

impl BuffMalzaharQ {
    pub fn new(silence_duration: f32, duration: f32) -> Self {
        Self {
            silence_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 马尔扎哈E - 恶兆之影（感染）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MalzaharE" })]
pub struct BuffMalzaharE {
    pub damage: f32,
    pub timer: Timer,
}

impl BuffMalzaharE {
    pub fn new(damage: f32, duration: f32) -> Self {
        Self {
            damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
