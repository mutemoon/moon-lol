use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 泰达米尔W - 嘲弄（减速+攻击力削减）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "TryndamereW" })]
pub struct BuffTryndamereW {
    pub slow_percent: f32,
    pub armor_reduction: f32,
    pub timer: Timer,
}

impl BuffTryndamereW {
    pub fn new(slow_percent: f32, armor_reduction: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            armor_reduction,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
