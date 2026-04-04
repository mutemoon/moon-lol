use bevy::prelude::*;

use crate::base::buff::Buff;

/// 魅惑效果
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Charm" })]
pub struct BuffCharm {
    pub timer: Timer,
}

impl BuffCharm {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 阿狸W技能 - 狐火
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AhriFoxFire" })]
pub struct BuffAhriFoxFire {
    pub remaining_flames: u8,
    pub timer: Timer,
}

impl BuffAhriFoxFire {
    pub fn new(flames: u8) -> Self {
        Self {
            remaining_flames: flames,
            timer: Timer::from_seconds(2.5, TimerMode::Once),
        }
    }
}

/// 阿狸被动 - 灵魂掠夺
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AhriPassive" })]
pub struct BuffAhriPassive {
    pub souls_collected: u8,
}

impl BuffAhriPassive {
    pub fn new() -> Self {
        Self { souls_collected: 0 }
    }
}
