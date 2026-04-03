use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 男枪E - 快速拔枪
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "GravesE" })]
pub struct BuffGravesE {
    pub stacks: u8,
    pub timer: Timer,
}

impl BuffGravesE {
    pub fn new() -> Self {
        Self {
            stacks: 1,
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}
