use bevy::prelude::*;
use crate::Buff;

/// Briar被动 - 赤红诅咒（流血）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "BriarPassive" })]
pub struct BuffBriarPassive {
    pub stacks: u8,
    pub damage_per_second: f32,
    pub timer: Timer,
}

impl BuffBriarPassive {
    pub fn new(stacks: u8, damage_per_second: f32, duration: f32) -> Self {
        Self {
            stacks,
            damage_per_second,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// Briar Q - 嗜血冲击（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "BriarQ" })]
pub struct BuffBriarQ {
    pub stun_duration: f32,
    pub armor_reduction: f32,
    pub timer: Timer,
}

impl BuffBriarQ {
    pub fn new(stun_duration: f32, armor_reduction: f32, duration: f32) -> Self {
        Self {
            stun_duration,
            armor_reduction,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// Briar W - 血之狂怒
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "BriarW" })]
pub struct BuffBriarW {
    pub attack_speed_bonus: f32,
    pub move_speed_bonus: f32,
    pub timer: Timer,
}

impl BuffBriarW {
    pub fn new(attack_speed_bonus: f32, move_speed_bonus: f32, duration: f32) -> Self {
        Self {
            attack_speed_bonus,
            move_speed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
