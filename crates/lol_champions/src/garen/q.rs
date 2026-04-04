use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 盖伦Q技能buff - 移动速度加成和下次攻击增强
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "GarenQ" })]
pub struct BuffGarenQ {
    /// 移动速度加成百分比 (e.g., 0.3 = 30%)
    pub move_speed_bonus: f32,
    /// 持续时间
    pub duration: f32,
    /// 计时器
    pub timer: Timer,
}

impl BuffGarenQ {
    pub fn new(move_speed_bonus: f32, duration: f32) -> Self {
        Self {
            move_speed_bonus,
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        self.timer.is_finished()
    }
}

/// 盖伦Q的下次攻击增强buff
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "GarenQAttack" })]
pub struct BuffGarenQAttack {
    /// 沉默持续时间
    pub silence_duration: f32,
}

impl BuffGarenQAttack {
    pub fn new(silence_duration: f32) -> Self {
        Self { silence_duration }
    }
}
