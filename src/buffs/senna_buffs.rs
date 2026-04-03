use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 赛娜W - 墨影缚（禁锢）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SennaW" })]
pub struct BuffSennaW {
    pub root_duration: f32,
    pub timer: Timer,
}

impl BuffSennaW {
    pub fn new(root_duration: f32, duration: f32) -> Self {
        Self {
            root_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
