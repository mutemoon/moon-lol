use bevy::prelude::*;
use crate::Buff;

/// 兰博W - 破碎护盾（护盾）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RumbleW" })]
pub struct BuffRumbleW {
    pub shield_amount: f32,
    pub timer: Timer,
}

impl BuffRumbleW {
    pub fn new(shield_amount: f32, duration: f32) -> Self {
        Self {
            shield_amount,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
