use bevy::prelude::*;
use crate::Buff;

/// 艾希Q - 集中火力
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AsheQ" })]
pub struct BuffAsheQ {
    pub timer: Timer,
}

impl BuffAsheQ {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(6.0, TimerMode::Once),
        }
    }
}
