use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 奥瑞利安被动 - 冬境之灵
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AuroraPassive" })]
pub struct BuffAuroraPassive {
    pub timer: Timer,
}

impl BuffAuroraPassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}

/// 奥瑞利安R - 极寒领域
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AuroraR" })]
pub struct BuffAuroraR {
    pub timer: Timer,
}

impl BuffAuroraR {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Once),
        }
    }
}
