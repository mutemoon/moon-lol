use bevy::prelude::*;

use crate::base::buff::Buff;

/// 瑟提Q - 强化下两次攻击 + 移速
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SettQ" })]
pub struct BuffSettQ {
    pub remaining_hits: u8,
    pub move_speed_bonus: f32,
    pub timer: Timer,
}

impl BuffSettQ {
    pub fn new(remaining_hits: u8, move_speed_bonus: f32, duration: f32) -> Self {
        Self {
            remaining_hits,
            move_speed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        self.timer.is_finished()
    }
}
