use bevy::prelude::*;
use crate::Buff;

/// 乐芙兰被动 - 镜像（低血量分身）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LeBlancMirrorImage" })]
pub struct BuffLeBlancMirrorImage {
    pub duration: f32,
    pub timer: Timer,
}

impl BuffLeBlancMirrorImage {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 乐芙兰Q - 恶意印记标记
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LeBlancQ" })]
pub struct BuffLeBlancQ {
    pub damage: f32,
    pub timer: Timer,
}

impl BuffLeBlancQ {
    pub fn new(damage: f32, duration: f32) -> Self {
        Self {
            damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 乐芙兰W - 扭曲（位移）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LeBlancW" })]
pub struct BuffLeBlancW {
    pub damage: f32,
    pub timer: Timer,
}

impl BuffLeBlancW {
    pub fn new(damage: f32, duration: f32) -> Self {
        Self {
            damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 乐芙兰E - 幻影锁链（禁锢）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LeBlancE" })]
pub struct BuffLeBlancE {
    pub damage: f32,
    pub root_duration: f32,
    pub timer: Timer,
}

impl BuffLeBlancE {
    pub fn new(damage: f32, root_duration: f32, duration: f32) -> Self {
        Self {
            damage,
            root_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
