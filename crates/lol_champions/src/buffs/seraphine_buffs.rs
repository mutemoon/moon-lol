use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 萨勒芬妮W - 汲取（护盾）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SeraphineW" })]
pub struct BuffSeraphineW {
    pub shield_amount: f32,
    pub timer: Timer,
}

impl BuffSeraphineW {
    pub fn new(shield_amount: f32, duration: f32) -> Self {
        Self {
            shield_amount,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 萨勒芬妮E - 聚音之墙（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SeraphineE" })]
pub struct BuffSeraphineE {
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffSeraphineE {
    pub fn new(stun_duration: f32, duration: f32) -> Self {
        Self {
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
