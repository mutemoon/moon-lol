use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 阿卡丽Q - 虎牙（持续伤害）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AkshanPassive" })]
pub struct BuffAkshanPassive {
    pub stacks: u8,
    pub damage: f32,
    pub timer: Timer,
}

impl BuffAkshanPassive {
    pub fn new(stacks: u8, damage: f32, duration: f32) -> Self {
        Self {
            stacks,
            damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
