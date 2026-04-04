use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 赫卡里姆Q技能层数buff - 叠层后减少Q冷却并增加伤害
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "HecarimQ" })]
pub struct BuffHecarimQ {
    /// 当前层数
    pub stacks: u8,
    /// 最大层数
    pub max_stacks: u8,
    /// 每层减少的冷却时间
    pub cooldown_reduction_per_stack: f32,
    /// 每层增加的伤害百分比
    pub damage_bonus_per_stack: f32,
}

impl BuffHecarimQ {
    pub fn new(
        max_stacks: u8,
        cooldown_reduction_per_stack: f32,
        damage_bonus_per_stack: f32,
    ) -> Self {
        Self {
            stacks: 1,
            max_stacks,
            cooldown_reduction_per_stack,
            damage_bonus_per_stack,
        }
    }

    pub fn add_stack(&mut self) -> bool {
        if self.stacks < self.max_stacks {
            self.stacks += 1;
            true
        } else {
            false
        }
    }

    /// 获取当前总伤害加成
    pub fn total_damage_bonus(&self) -> f32 {
        (self.stacks as f32 - 1.0) * self.damage_bonus_per_stack
    }

    /// 获取当前冷却减免
    pub fn total_cooldown_reduction(&self) -> f32 {
        (self.stacks as f32 - 1.0) * self.cooldown_reduction_per_stack
    }
}

/// 赫卡里姆W - 灵魂收割，持续时间内造成伤害并治疗
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "HecarimW" })]
pub struct BuffHecarimW {
    pub duration: f32,
    pub timer: Timer,
}

impl BuffHecarimW {
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
