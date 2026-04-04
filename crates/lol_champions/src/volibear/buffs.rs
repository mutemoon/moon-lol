use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 沃利贝尔Q - 加速 + 下次攻击眩晕
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "VolibearQ" })]
pub struct BuffVolibearQ {
    pub move_speed_bonus: f32,
    pub stun_duration: f32,
    pub duration: f32,
    pub timer: Timer,
}

impl BuffVolibearQ {
    pub fn new(move_speed_bonus: f32, stun_duration: f32, duration: f32) -> Self {
        Self {
            move_speed_bonus,
            stun_duration,
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        self.timer.is_finished()
    }
}

/// 沃利贝尔W标记 —— 挂在被W1命中的目标上
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "VolibearWMark" })]
pub struct DebuffVolibearWMark {
    pub source: Entity, // 沃利贝尔自身
    pub timer: Timer,
}

impl DebuffVolibearWMark {
    pub fn new(source: Entity, duration: f32) -> Self {
        Self {
            source,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
