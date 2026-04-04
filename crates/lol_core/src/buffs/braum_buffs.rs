use bevy::prelude::*;

use crate::base::buff::Buff;

/// 布隆被动 - 震荡猛击
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "BraumPassive" })]
pub struct BuffBraumPassive {
    pub stacks: u8,
}

impl BuffBraumPassive {
    pub fn new() -> Self {
        Self { stacks: 1 }
    }
}

/// 布隆W - 挺身而出
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "BraumW" })]
pub struct BuffBraumW {
    pub timer: Timer,
}

impl BuffBraumW {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(4.0, TimerMode::Once),
        }
    }
}
