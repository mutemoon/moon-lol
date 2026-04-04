use bevy::prelude::*;

use crate::base::buff::Buff;

/// 索拉卡E - 星界隔绝（沉默）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SorakaE" })]
pub struct BuffSorakaE {
    pub silence_duration: f32,
    pub timer: Timer,
}

impl BuffSorakaE {
    pub fn new(silence_duration: f32, duration: f32) -> Self {
        Self {
            silence_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
