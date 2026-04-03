use bevy::prelude::*;
use crate::Buff;

/// 俄洛伊被动 - 夺命者的预言
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "IllaoiPassive" })]
pub struct BuffIllaoiPassive {
    pub timer: Timer,
}

impl BuffIllaoiPassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(8.0, TimerMode::Once),
        }
    }
}
