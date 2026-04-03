use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 希瓦娜E - 龙牙突袭（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "ShyvanaE" })]
pub struct BuffShyvanaE {
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffShyvanaE {
    pub fn new(stun_duration: f32, duration: f32) -> Self {
        Self {
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
