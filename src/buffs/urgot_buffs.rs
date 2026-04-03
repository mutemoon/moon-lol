use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 厄加特W技能buff - 开启期间自动攻击周围敌人
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "UrgotW" })]
pub struct BuffUrgotW {
    /// 攻击间隔（秒）
    pub attack_interval: f32,
    /// 当前已用时间
    pub elapsed: f32,
    /// 移动速度降低百分比
    pub move_speed_reduction: f32,
    /// 最大攻击距离
    pub max_range: f32,
}

impl BuffUrgotW {
    pub fn new(attack_interval: f32, move_speed_reduction: f32, max_range: f32) -> Self {
        Self {
            attack_interval,
            elapsed: 0.0,
            move_speed_reduction,
            max_range,
        }
    }

    pub fn tick(&mut self, delta: f32) {
        self.elapsed += delta;
    }

    pub fn should_attack(&self) -> bool {
        self.elapsed >= self.attack_interval
    }

    pub fn reset_timer(&mut self) {
        self.elapsed = 0.0;
    }
}

/// 厄加特R 斩杀标记 —— 挂在被R命中的目标上
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "UrgotR" })]
pub struct DebuffUrgotR {
    pub source: Entity, // 厄加特自身
    pub timer: Timer,
}

impl DebuffUrgotR {
    pub fn new(source: Entity, duration: f32) -> Self {
        Self {
            source,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
