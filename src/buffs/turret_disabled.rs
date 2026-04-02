use bevy::prelude::*;

use crate::Buff;

/// 防御塔禁用 —— 挂在被禁用的防御塔上
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "TurretDisabled" })]
pub struct BuffTurretDisabled {
    pub timer: Timer,
}

impl BuffTurretDisabled {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
