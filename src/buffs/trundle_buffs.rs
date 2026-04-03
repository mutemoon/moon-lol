use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 茂凯Q - 荆棘缠绕（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "TrundleQ" })]
pub struct BuffTrundleQ {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffTrundleQ {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
