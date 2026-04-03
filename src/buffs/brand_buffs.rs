use bevy::prelude::*;
use crate::Buff;

/// 布兰德被动 - 炽燃之焰
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "BrandPassive" })]
pub struct BuffBrandPassive {
    pub stacks: u8,
    pub timer: Timer,
}

impl BuffBrandPassive {
    pub fn new() -> Self {
        Self {
            stacks: 1,
            timer: Timer::from_seconds(4.0, TimerMode::Once),
        }
    }
}
