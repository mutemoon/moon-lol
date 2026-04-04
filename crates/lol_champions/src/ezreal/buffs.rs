use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 伊泽瑞尔被动 - 咒能高涨
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "EzrealPassive" })]
pub struct BuffEzrealPassive {
    pub stacks: u8,
    pub timer: Timer,
}

impl BuffEzrealPassive {
    pub fn new() -> Self {
        Self {
            stacks: 1,
            timer: Timer::from_seconds(6.0, TimerMode::Once),
        }
    }
}
