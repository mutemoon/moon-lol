use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 塔里克E - 正义荣耀（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "TaricE" })]
pub struct BuffTaricE {
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffTaricE {
    pub fn new(stun_duration: f32, duration: f32) -> Self {
        Self {
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
