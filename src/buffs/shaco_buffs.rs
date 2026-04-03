use bevy::prelude::*;
use crate::Buff;

/// 萨科W - 幻痛（恐惧）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "ShacoW" })]
pub struct BuffShacoW {
    pub fear_duration: f32,
    pub timer: Timer,
}

impl BuffShacoW {
    pub fn new(fear_duration: f32, duration: f32) -> Self {
        Self {
            fear_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
