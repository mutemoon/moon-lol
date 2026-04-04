use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 杰斯被动 - 雷霆一击
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "JaycePassive" })]
pub struct BuffJaycePassive {
    pub timer: Timer,
}

impl BuffJaycePassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        }
    }
}
