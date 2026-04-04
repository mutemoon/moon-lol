use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 潘森被动计数器 - 每3次强化下一个技能
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "PantheonPassiveStacks" })]
pub struct BuffPantheonPassive {
    pub stacks: u8,
}

/// 潘森E - 盾牌格挡
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "PantheonE" })]
pub struct BuffPantheonE {
    pub direction: Vec2,
    pub duration: f32,
    pub timer: Timer,
}

impl BuffPantheonE {
    pub fn new(direction: Vec2, duration: f32) -> Self {
        Self {
            direction,
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        self.timer.is_finished()
    }
}
