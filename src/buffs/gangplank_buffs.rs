use bevy::prelude::*;
use crate::Buff;

/// 普朗克被动 - 火药试炼
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "GangplankPassive" })]
pub struct BuffGangplankPassive {
    pub timer: Timer,
}

impl BuffGangplankPassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        }
    }
}
