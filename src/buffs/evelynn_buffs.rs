use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 伊芙琳被动 - 恶魔之影
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "EvelynnPassive" })]
pub struct BuffEvelynnPassive {
    pub timer: Timer,
}

impl BuffEvelynnPassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(4.0, TimerMode::Once),
        }
    }
}
