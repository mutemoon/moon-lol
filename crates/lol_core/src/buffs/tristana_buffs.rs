use bevy::prelude::*;

use crate::base::buff::Buff;

/// 崔丝塔娜W - 火箭跳跃（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "TristanaW" })]
pub struct BuffTristanaW {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffTristanaW {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
