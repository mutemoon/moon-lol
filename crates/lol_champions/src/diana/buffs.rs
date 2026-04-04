use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 黛安娜被动 - 银光刃
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "DianaPassive" })]
pub struct BuffDianaPassive {
    pub timer: Timer,
}

impl BuffDianaPassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}
