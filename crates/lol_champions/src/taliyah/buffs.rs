use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 塔莉垭W - 伍图突岩（击飞）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "TaliyahW" })]
pub struct BuffTaliyahW {
    pub knockup_duration: f32,
    pub timer: Timer,
}

impl BuffTaliyahW {
    pub fn new(knockup_duration: f32, duration: f32) -> Self {
        Self {
            knockup_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
