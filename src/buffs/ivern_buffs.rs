use bevy::prelude::*;
use crate::Buff;

/// 艾翁被动 - 森林之友
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "IvernPassive" })]
pub struct BuffIvernPassive {
    pub timer: Timer,
}

impl BuffIvernPassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(6.0, TimerMode::Once),
        }
    }
}
