use bevy::prelude::*;

use crate::base::buff::Buff;

/// 贾克斯E技能buff - 闪避
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "JaxE" })]
pub struct BuffJaxE {
    /// 闪避持续时间
    pub duration: f32,
    /// 闪避概率 (0.0 - 1.0)
    pub dodge_chance: f32,
    /// 周围敌人被攻击的概率
    pub aoe_dodge_chance: f32,
    /// 计时器
    pub timer: Timer,
}

impl BuffJaxE {
    pub fn new(duration: f32, dodge_chance: f32, aoe_dodge_chance: f32) -> Self {
        Self {
            duration,
            dodge_chance,
            aoe_dodge_chance,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        self.timer.is_finished()
    }
}
