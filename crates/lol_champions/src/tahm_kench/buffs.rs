use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 塔姆E - 厚实表皮（护盾）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "TahmKenchE" })]
pub struct BuffTahmKenchE {
    pub shield_amount: f32,
    pub timer: Timer,
}

impl BuffTahmKenchE {
    pub fn new(shield_amount: f32, duration: f32) -> Self {
        Self {
            shield_amount,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
