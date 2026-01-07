use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{BuffDamageReduction, BuffShieldMagic, BuffShieldWhite, Buffs, Health};

/// 伤害系统插件
#[derive(Default)]
pub struct PluginDamage;

impl Plugin for PluginDamage {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_damage_create);
    }
}

#[derive(Component, Reflect, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Damage(pub f32);

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Armor(pub f32);

/// 伤害类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    /// 物理伤害
    Physical,
    /// 魔法伤害
    Magic,
    /// 真实伤害（无视所有防御）
    True,
}

/// 伤害事件，包含伤害来源、目标、伤害类型和数值
#[derive(EntityEvent, Debug)]
pub struct CommandDamageCreate {
    pub entity: Entity,
    /// 伤害来源实体
    pub source: Entity,
    /// 伤害类型
    pub damage_type: DamageType,
    /// 伤害数值
    pub amount: f32,
}

#[derive(EntityEvent, Debug)]
pub struct EventDamageCreate {
    pub entity: Entity,
    pub source: Entity,
    pub damage_type: DamageType,
    pub damage_result: DamageResult,
}

/// 伤害计算结果
#[derive(Debug)]
pub struct DamageResult {
    /// 最终造成的伤害
    pub final_damage: f32,
    /// 被白色护盾吸收的伤害
    pub white_shield_absorbed: f32,
    /// 被魔法护盾吸收的伤害
    pub magic_shield_absorbed: f32,
    /// 被减免的伤害
    pub reduced_damage: f32,
    /// 被护甲减免的伤害
    pub armor_reduced_damage: f32,
    /// 原始伤害
    pub original_damage: f32,
}

/// 伤害系统 - 处理伤害事件
pub fn on_command_damage_create(
    trigger: On<CommandDamageCreate>,
    mut commands: Commands,
    mut query: Query<(&mut Health, Option<&Armor>, Option<&Buffs>)>,
    mut q_shield_white: Query<&mut BuffShieldWhite>,
    mut q_shield_magic: Query<&mut BuffShieldMagic>,
    q_damage_reduction: Query<&BuffDamageReduction>,
) {
    debug!(
        "{:?} 对 {:?} 造成 {:.1} 点 {:?} 伤害",
        trigger.source,
        trigger.event_target(),
        trigger.amount,
        trigger.damage_type,
    );

    let Ok((mut health, armor, buffs)) = query.get_mut(trigger.event_target()) else {
        debug!("未找到伤害目标实体 {:?}", trigger.event_target());
        return;
    };

    let health_before = health.value;
    let armor_value = armor.map(|a| a.0);

    let mut remaining_damage = trigger.amount;
    let mut white_shield_absorbed = 0.0;
    let mut magic_shield_absorbed = 0.0;
    let mut reduced_damage = 0.0;
    let mut armor_reduced_damage = 0.0;

    // 真实伤害无视所有防御机制
    if trigger.damage_type == DamageType::True {
        health.value -= remaining_damage;
    } else {
        // 对物理伤害应用护甲减伤
        if trigger.damage_type == DamageType::Physical {
            if let Some(armor_val) = armor_value {
                if armor_val > 0.0 {
                    let damage_after_armor = remaining_damage * 100.0 / (100.0 + armor_val);
                    armor_reduced_damage = remaining_damage - damage_after_armor;
                    remaining_damage = damage_after_armor;
                }
            }
        }

        // 应用伤害减免buff
        if let Some(target_buffs) = buffs {
            let mut total_reduction = 0.0;
            for buff_entity in target_buffs.iter() {
                if let Ok(reduction) = q_damage_reduction.get(buff_entity) {
                    if reduction.applies_to(trigger.damage_type) {
                        // 使用乘法叠加公式：总减免 = 1 - (1 - r1) * (1 - r2) * ...
                        total_reduction =
                            1.0 - (1.0 - total_reduction) * (1.0 - reduction.percentage);
                    }
                }
            }
            if total_reduction > 0.0 {
                let reduction_amount = remaining_damage * total_reduction;
                reduced_damage = reduction_amount;
                remaining_damage -= reduction_amount;
            }
        }

        // 应用护盾（白色护盾优先）
        if let Some(target_buffs) = buffs {
            for buff_entity in target_buffs.iter() {
                if let Ok(mut shield) = q_shield_white.get_mut(buff_entity) {
                    let before = remaining_damage;
                    remaining_damage = shield.absorb_damage(remaining_damage);
                    white_shield_absorbed += before - remaining_damage;
                    if remaining_damage <= 0.0 {
                        break;
                    }
                }
            }
        }

        // 如果是魔法伤害且还有剩余伤害，应用魔法护盾
        if trigger.damage_type == DamageType::Magic && remaining_damage > 0.0 {
            if let Some(target_buffs) = buffs {
                for buff_entity in target_buffs.iter() {
                    if let Ok(mut shield) = q_shield_magic.get_mut(buff_entity) {
                        let before = remaining_damage;
                        remaining_damage = shield.absorb_magic_damage(remaining_damage);
                        magic_shield_absorbed += before - remaining_damage;
                        if remaining_damage <= 0.0 {
                            break;
                        }
                    }
                }
            }
        }

        health.value -= remaining_damage;
    }

    let result = DamageResult {
        final_damage: remaining_damage,
        white_shield_absorbed,
        magic_shield_absorbed,
        reduced_damage,
        armor_reduced_damage,
        original_damage: trigger.amount,
    };

    debug!(
        "伤害已应用 {:?} -> {:?} 类型 {:?} 原始伤害 {:.1} 最终伤害 {:.1} 生命值 {:.1} -> {:.1} 护甲减免 {:.1} 白盾吸收 {:.1} 魔盾吸收 {:.1} 减伤 {:.1}",
        trigger.source,
        trigger.event_target(),
        trigger.damage_type,
        result.original_damage,
        result.final_damage,
        health_before,
        health.value,
        result.armor_reduced_damage,
        result.white_shield_absorbed,
        result.magic_shield_absorbed,
        result.reduced_damage
    );

    commands.trigger(EventDamageCreate {
        entity: trigger.event_target(),
        source: trigger.source,
        damage_type: trigger.damage_type,
        damage_result: result,
    });

    if health.value <= 0.0 {
        debug!(
            "{:?} 生命值降至 {:.1} 已达到死亡阈值",
            trigger.event_target(),
            health.value
        );
    }
}
