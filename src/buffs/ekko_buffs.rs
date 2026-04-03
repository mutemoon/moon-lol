use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 艾克被动 - Z型驱动共振
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "EkkoPassive" })]
pub struct BuffEkkoPassive {
    pub stacks: u8,
    pub timer: Timer,
}

impl BuffEkkoPassive {
    pub fn new() -> Self {
        Self {
            stacks: 1,
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}
