use bevy::prelude::*;

use crate::Buff;

#[derive(Default)]
pub struct PluginShieldWhite;

impl Plugin for PluginShieldWhite {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_shield_white);
    }
}

/// 白色护盾组件 - 可以抵挡所有类型的伤害
#[derive(Component, Debug, Default, Clone)]
#[require(Buff = Buff { name: "ShieldWhite" })]
pub struct BuffShieldWhite {
    /// 当前护盾值
    pub current: f32,
    /// 最大护盾值
    pub max: f32,
}

impl BuffShieldWhite {
    pub fn new(amount: f32) -> Self {
        Self {
            current: amount,
            max: amount,
        }
    }

    /// 吸收伤害，返回剩余伤害
    pub fn absorb_damage(&mut self, damage: f32) -> f32 {
        let absorbed = damage.min(self.current);
        self.current -= absorbed;
        damage - absorbed
    }

    /// 检查护盾是否已耗尽
    pub fn is_depleted(&self) -> bool {
        self.current <= 0.0
    }
}

fn update_shield_white(mut commands: Commands, q_shield: Query<(Entity, &BuffShieldWhite)>) {
    for (entity, shield) in q_shield.iter() {
        if shield.is_depleted() {
            debug!("正在移除实体 {:?} 的耗尽白色护盾", entity);
            commands.entity(entity).despawn();
        }
    }
}
