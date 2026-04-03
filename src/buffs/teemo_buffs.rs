use bevy::prelude::*;
use crate::Buff;

/// 提莫Q - 致盲（致盲）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "TeemoQ" })]
pub struct BuffTeemoQ {
    pub blind_duration: f32,
    pub timer: Timer,
}

impl BuffTeemoQ {
    pub fn new(blind_duration: f32, duration: f32) -> Self {
        Self {
            blind_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
