use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 图奇被动 - 致命毒液（持续伤害）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "TwitchPassive" })]
pub struct BuffTwitchPassive {
    pub stacks: u8,
    pub damage_per_second: f32,
    pub timer: Timer,
}

impl BuffTwitchPassive {
    pub fn new(stacks: u8, damage_per_second: f32, duration: f32) -> Self {
        Self {
            stacks,
            damage_per_second,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 图奇W - 毒瓶（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "TwitchW" })]
pub struct BuffTwitchW {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffTwitchW {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
