use bevy::prelude::*;

use crate::base::buff::Buff;

/// 牛头被动 - 胜利怒吼
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AlistarPassive" })]
pub struct BuffAlistarPassive {
    pub stacks: u8,
}

impl BuffAlistarPassive {
    pub fn new() -> Self {
        Self { stacks: 0 }
    }
}

/// 牛头R - 坚定意志（伤害减免）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AlistarR" })]
pub struct BuffAlistarR {
    pub timer: Timer,
}

impl BuffAlistarR {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(7.0, TimerMode::Once),
        }
    }
}
