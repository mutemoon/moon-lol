use bevy::prelude::*;
use crate::Buff;

/// 金克丝被动 - 超活跃状态
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "JinxExcited" })]
pub struct BuffJinxExcited {
    pub movespeed_bonus: f32,
    pub attackspeed_bonus: f32,
    pub stacks: u8,
    pub timer: Timer,
}

impl BuffJinxExcited {
    pub fn new(duration: f32) -> Self {
        Self {
            movespeed_bonus: 1.75,
            attackspeed_bonus: 0.25,
            stacks: 1,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn add_stack(&mut self) {
        if self.stacks < 5 {
            self.stacks += 1;
        }
    }
}

/// 金克丝Q - 砰砰（机枪）攻速加成
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "JinxQPowPow" })]
pub struct BuffJinxQPowPow {
    pub attackspeed_bonus: f32,
    pub stacks: u8,
    pub timer: Timer,
}

impl BuffJinxQPowPow {
    pub fn new(attackspeed_bonus: f32, duration: f32) -> Self {
        Self {
            attackspeed_bonus,
            stacks: 1,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn add_stack(&mut self) {
        if self.stacks < 3 {
            self.stacks += 1;
        }
    }
}

/// 金克丝W - 电击弹减速
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "JinxW" })]
pub struct BuffJinxW {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffJinxW {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
