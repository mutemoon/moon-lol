use bevy::prelude::*;

use crate::Buff;

/// 诺手被动 - 出血标记，最多5层
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "DariusBleed" })]
pub struct BuffDariusBleed {
    pub stacks: u8,
    pub timer: Timer,
}

impl BuffDariusBleed {
    pub fn new(stacks: u8, duration: f32) -> Self {
        Self {
            stacks,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        self.timer.is_finished()
    }
}

/// 诺手W - 强化下次攻击 + 减速
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "DariusW" })]
pub struct BuffDariusW {
    pub slow_percent: f32,
    pub duration: f32,
    pub timer: Timer,
}

impl BuffDariusW {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        self.timer.is_finished()
    }
}
