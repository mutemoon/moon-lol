use bevy::prelude::*;

use crate::{Buff, DamageType};

#[derive(Default)]
pub struct PluginDamageReduction;

impl Plugin for PluginDamageReduction {
    fn build(&self, _app: &mut App) {}
}

/// 伤害减免buff组件
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "DamageReduction" })]
pub struct BuffDamageReduction {
    /// 减免百分比 (0.0 - 1.0)
    pub percentage: f32,
    /// 减免的伤害类型，None表示对所有类型有效
    pub damage_type: Option<DamageType>,
}

impl BuffDamageReduction {
    pub fn new(percentage: f32, damage_type: Option<DamageType>) -> Self {
        Self {
            percentage: percentage.clamp(0.0, 1.0),
            damage_type,
        }
    }

    /// 检查buff是否对指定伤害类型有效
    pub fn applies_to(&self, damage_type: DamageType) -> bool {
        self.damage_type.map_or(true, |dt| dt == damage_type)
    }
}
