use bevy::prelude::*;
use crate::Buff;

/// 派克Q - 骨齿穿刺（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "PykeQ" })]
pub struct BuffPykeQ {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffPykeQ {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 派克E - 幻影潜行（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "PykeE" })]
pub struct BuffPykeE {
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffPykeE {
    pub fn new(stun_duration: f32, duration: f32) -> Self {
        Self {
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 派克R - 水银深渊（斩杀标记）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "PykeR" })]
pub struct BuffPykeR {
    pub execute_threshold: f32,
    pub timer: Timer,
}

impl BuffPykeR {
    pub fn new(execute_threshold: f32, duration: f32) -> Self {
        Self {
            execute_threshold,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
