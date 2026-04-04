use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 冰凤R - 冰川风暴
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AniviaR" })]
pub struct BuffAniviaR {
    pub timer: Timer,
}

impl BuffAniviaR {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(12.0, TimerMode::Once),
        }
    }
}
