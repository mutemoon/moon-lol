use bevy::prelude::*;

use crate::base::buff::Buff;

/// 蒸汽机器人被动 - 法力护盾
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "BlitzcrankPassive" })]
pub struct BuffBlitzcrankPassive {
    pub timer: Timer,
}

impl BuffBlitzcrankPassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(10.0, TimerMode::Once),
        }
    }
}

/// 蒸汽机器人W - 过载运转
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "BlitzcrankW" })]
pub struct BuffBlitzcrankW {
    pub timer: Timer,
}

impl BuffBlitzcrankW {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(8.0, TimerMode::Once),
        }
    }
}
