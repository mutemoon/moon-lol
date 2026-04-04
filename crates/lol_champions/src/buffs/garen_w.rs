use bevy::prelude::*;
use lol_core::base::buff::Buff;
use lol_core::damage::DamageType;

/// 盖伦W技能buff - 韧性和伤害减免
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "GarenW" })]
pub struct BuffGarenW {
    /// 韧性加成百分比 (e.g., 0.3 = 30%)
    pub tenacity: f32,
    /// 伤害减免百分比 (e.g., 0.2 = 20%)
    pub damage_reduction: f32,
    /// 护盾值
    pub shield: f32,
    /// 持续时间
    pub duration: f32,
    /// 计时器
    pub timer: Timer,
}

impl BuffGarenW {
    pub fn new(tenacity: f32, damage_reduction: f32, shield: f32, duration: f32) -> Self {
        Self {
            tenacity,
            damage_reduction,
            shield,
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        self.timer.is_finished()
    }

    /// 检查buff是否对指定伤害类型有效
    pub fn applies_to(&self, _damage_type: DamageType) -> bool {
        true // 对所有伤害类型有效
    }
}
