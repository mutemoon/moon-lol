use bevy::prelude::*;

use crate::base::buff::Buff;

/// 茂凯被动 - 吸元术
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MaokaiSapMagic" })]
pub struct BuffMaokaiSapMagic {
    pub stacks: u8,
    pub damage: f32,
    pub timer: Timer,
}

impl BuffMaokaiSapMagic {
    pub fn new(stacks: u8, damage: f32, duration: f32) -> Self {
        Self {
            stacks,
            damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 茂凯W - 扭曲突刺（禁锢）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MaokaiW" })]
pub struct BuffMaokaiW {
    pub root_duration: f32,
    pub timer: Timer,
}

impl BuffMaokaiW {
    pub fn new(root_duration: f32, duration: f32) -> Self {
        Self {
            root_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 茂凯E - 滚动荆棘（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MaokaiE" })]
pub struct BuffMaokaiE {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffMaokaiE {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
