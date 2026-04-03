use bevy::prelude::*;
use crate::Buff;

/// 琴女W - 迅奏鸣曲（护盾）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SonaW" })]
pub struct BuffSonaW {
    pub shield_amount: f32,
    pub timer: Timer,
}

impl BuffSonaW {
    pub fn new(shield_amount: f32, duration: f32) -> Self {
        Self {
            shield_amount,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 琴女E -  crescino（移速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SonaE" })]
pub struct BuffSonaE {
    pub movespeed_bonus: f32,
    pub timer: Timer,
}

impl BuffSonaE {
    pub fn new(movespeed_bonus: f32, duration: f32) -> Self {
        Self {
            movespeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
