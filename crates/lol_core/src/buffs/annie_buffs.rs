use bevy::prelude::*;

use crate::base::buff::Buff;

/// 安妮被动 - 嗜火
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AnniePassive" })]
pub struct BuffAnniePassive {
    pub stacks: u8,
}

impl BuffAnniePassive {
    pub fn new() -> Self {
        Self { stacks: 0 }
    }

    pub fn increment() -> Self {
        Self { stacks: 1 }
    }
}

/// 安妮E - 熔岩护盾
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AnnieShield" })]
pub struct BuffAnnieShield {
    pub timer: Timer,
}

impl BuffAnnieShield {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}
