use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 奥拉夫W技能buff - 攻速加成和护盾
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "OlafW" })]
pub struct BuffOlafW {
    /// 攻速加成百分比 (e.g., 0.4 = 40%)
    pub attack_speed_bonus: f32,
    /// 护盾值
    pub shield: f32,
    /// 持续时间
    pub duration: f32,
    /// 计时器
    pub timer: Timer,
}

impl BuffOlafW {
    pub fn new(attack_speed_bonus: f32, shield: f32, duration: f32) -> Self {
        Self {
            attack_speed_bonus,
            shield,
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        self.timer.is_finished()
    }
}

/// 奥拉夫R技能buff - 免疫控制效果
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "OlafR" })]
pub struct BuffOlafR {
    /// 免疫控制持续时间
    pub duration: f32,
    /// 计时器
    pub timer: Timer,
}

impl BuffOlafR {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        self.timer.is_finished()
    }
}
