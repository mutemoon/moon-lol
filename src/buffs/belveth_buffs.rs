use bevy::prelude::*;
use crate::Buff;

///  Bel'Veth 被动 - 死亡之紫
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "BelvethPassive" })]
pub struct BuffBelvethPassive {
    pub stacks: u8,
    pub attack_speed_bonus: f32,
    pub timer: Timer,
}

impl BuffBelvethPassive {
    pub fn new(stacks: u8, attack_speed_bonus: f32, duration: f32) -> Self {
        Self {
            stacks,
            attack_speed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

///  Bel'Veth W - 天翻地覆（击飞+减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "BelvethW" })]
pub struct BuffBelvethW {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffBelvethW {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
