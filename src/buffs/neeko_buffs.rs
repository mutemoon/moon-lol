use bevy::prelude::*;
use crate::Buff;

/// 妮蔻被动 - 先天魅力（伪装）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NeekoPassive" })]
pub struct BuffNeekoPassive {
    pub disguised: bool,
    pub timer: Timer,
}

impl BuffNeekoPassive {
    pub fn new(disguised: bool, duration: f32) -> Self {
        Self {
            disguised,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 妮蔻E - 纠缠之刺（禁锢）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NeekoE" })]
pub struct BuffNeekoE {
    pub root_duration: f32,
    pub timer: Timer,
}

impl BuffNeekoE {
    pub fn new(root_duration: f32, duration: f32) -> Self {
        Self {
            root_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 妮蔻R - 绽放（击飞和眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "NeekoR" })]
pub struct BuffNeekoR {
    pub damage: f32,
    pub knockup_duration: f32,
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffNeekoR {
    pub fn new(damage: f32, knockup_duration: f32, stun_duration: f32, duration: f32) -> Self {
        Self {
            damage,
            knockup_duration,
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
