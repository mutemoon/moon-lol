use bevy::prelude::*;
use crate::Buff;

/// 菲兹被动 - 灵活战士
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "FizzPassive" })]
pub struct BuffFizzPassive {
    pub timer: Timer,
}

impl BuffFizzPassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(6.0, TimerMode::Once),
        }
    }
}
