use bevy::prelude::*;

use crate::Buff;

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
    pub fn new(max_stacks: u8, cooldown_reduction_per_stack: f32, damage_bonus_per_stack: f32) -> Self {
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
