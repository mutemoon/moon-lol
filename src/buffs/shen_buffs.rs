use bevy::prelude::*;
use crate::Buff;

/// 慎W - 魂佑（闪避）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "ShenW" })]
pub struct BuffShenW {
    pub dodge_chance: f32,
    pub timer: Timer,
}

impl BuffShenW {
    pub fn new(dodge_chance: f32, duration: f32) -> Self {
        Self {
            dodge_chance,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
