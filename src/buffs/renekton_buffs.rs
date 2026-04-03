use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 鳄鱼R - 统治/变身
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RenektonR" })]
pub struct BuffRenektonR {
    pub bonus_health: f32,
    pub fury_per_second: f32,
    pub duration: f32,
    pub timer: Timer,
}

impl BuffRenektonR {
    pub fn new(bonus_health: f32, fury_per_second: f32, duration: f32) -> Self {
        Self {
            bonus_health,
            fury_per_second,
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        self.timer.is_finished()
    }
}
