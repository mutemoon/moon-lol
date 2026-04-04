use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 黑默丁格被动 - 科技亲和
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "HeimerPassive" })]
pub struct BuffHeimerPassive {
    pub timer: Timer,
}

impl BuffHeimerPassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}
