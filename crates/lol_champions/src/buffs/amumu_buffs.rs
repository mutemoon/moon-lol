use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 阿木木被动 - 诅咒之触
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AmumuPassive" })]
pub struct BuffAmumuPassive {
    pub timer: Timer,
}

impl BuffAmumuPassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}

/// 阿木木R - 木乃伊之咒
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AmumuR" })]
pub struct BuffAmumuR {
    pub timer: Timer,
}

impl BuffAmumuR {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}
