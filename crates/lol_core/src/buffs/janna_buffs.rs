use bevy::prelude::*;

use crate::base::buff::Buff;

/// 风暴之怒被动 - 顺风而行
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "JannaPassive" })]
pub struct BuffJannaPassive {
    pub timer: Timer,
}

impl BuffJannaPassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}
