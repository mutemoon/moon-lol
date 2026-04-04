use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 奇亚娜W - 元素之怒（草丛：隐身+移速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "QiyanaW" })]
pub struct BuffQiyanaW {
    pub movespeed_bonus: f32,
    pub timer: Timer,
}

impl BuffQiyanaW {
    pub fn new(movespeed_bonus: f32, duration: f32) -> Self {
        Self {
            movespeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 奇亚娜W - 元素之怒（河道：禁锢+减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "QiyanaWRoot" })]
pub struct BuffQiyanaWRoot {
    pub root_duration: f32,
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffQiyanaWRoot {
    pub fn new(root_duration: f32, slow_percent: f32, duration: f32) -> Self {
        Self {
            root_duration,
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
