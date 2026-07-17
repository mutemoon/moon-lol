//! Mordekaiser E - 断魂一拽 (Death's Grasp)
//!
//! 朝施法方向释放死亡之爪，将前方锥形（全角 100°）内的敌人向莫德凯撒拽回 250，
//! 并造成魔法伤害（基础 + 40% AP）。命中叠 1 层被动 Darkness（由被动观察者统一处理）。
//!
//! 组合表达（位移框架组合化）：
//! - `Cone{max_distance, 100°}` + `PullToward{pull_distance}` + `Damage{魔法}`
//!
//! 被动法术穿透（ron `MagicPen` 0.025-0.175）待法穿系统支持后实现。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::displace::{
    ActionDisplace, DisplaceEffect, DisplaceMotion, DisplaceTargetSelection,
};
use lol_core::damage::{AbilityPower, DamageType};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_data_value, get_skill_value};

use crate::mordekaiser::Mordekaiser;

/// E 锥形半径（ron `MaxDistance` = 550）
pub const MORDE_E_RANGE: f32 = 550.0;
/// E 锥形全角（度，ron 未单独提供，wiki 100°）
pub const MORDE_E_ANGLE: f32 = 100.0;
/// E 拽回距离（ron `KnockTowardsDistance` = 250）
pub const MORDE_E_PULL_DISTANCE: f32 = 250.0;
/// E 拽回速度
pub const MORDE_E_PULL_SPEED: f32 = 1500.0;

pub fn on_mordekaiser_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
    q_transform: Query<&Transform>,
    q_ap: Query<&AbilityPower>,
) {
    let entity = trigger.event_target();
    if q_morde.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    let Ok(transform) = q_transform.get(entity) else {
        return;
    };

    let pos = transform.translation.xz();
    let forward = (trigger.point - pos).normalize_or_zero();
    let forward = if forward == Vec2::ZERO {
        transform.forward().xz()
    } else {
        forward
    };

    let max_distance =
        get_skill_data_value(spell_obj, "MaxDistance", skill.level).unwrap_or(MORDE_E_RANGE);
    let pull_distance = get_skill_data_value(spell_obj, "KnockTowardsDistance", skill.level)
        .unwrap_or(MORDE_E_PULL_DISTANCE);
    let ap = q_ap.get(entity).map(|a| a.0).unwrap_or(0.0);
    let damage = get_skill_value(spell_obj, "total_damage", skill.level, |stat| {
        if stat == 0 { ap } else { 0.0 }
    })
    .unwrap_or(0.0);

    // 使用统一位移体系：锥形拽回 + 魔法伤害（tag:None 让被动观察者叠层）
    commands.trigger(ActionDisplace {
        entity,
        targets: DisplaceTargetSelection::Cone {
            range: max_distance,
            angle: MORDE_E_ANGLE,
            direction: forward,
        },
        motion: DisplaceMotion::PullToward {
            distance: pull_distance,
            speed: MORDE_E_PULL_SPEED,
        },
        effects: vec![DisplaceEffect::Damage {
            amount: damage,
            damage_type: DamageType::Magic,
            tag: None,
        }],
        cone_hit_policy: None,
    });

    debug!("莫德凯撒 E 断魂一拽：使用 ActionDisplace 锥形拽回");
}