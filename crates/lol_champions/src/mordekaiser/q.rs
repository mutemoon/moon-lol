//! Mordekaiser Q - 破灭之锤 (Obliterate)
//!
//! 向前挥砸权杖，对矩形区域（起始 400 / 长度 625 / 宽度 160）内敌人造成魔法伤害，
//! 孤立目标（区域内仅 1 个）额外乘算 IsolationScalar。
//!
//! 组合表达：
//! - 形状 = `DamageShape::Rectangle`（从 ron 读取 MaceStartDistance/MaceLength/RectangleWidth）
//! - 聚合条件 = `DamageModifier::Isolation`（区域内仅 1 目标时乘 IsolationScalar）
//! - 延迟 + 三阶段视觉 = `ActionDelayedDamage`（Delay->Impact->Fade）

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL1;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamageEffect, DamageModifier, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::delayed_damage::{ActionDelayedDamage, AoEIndicator, AoEOrigin};
use lol_core::damage::DamageType;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_data_value};

use crate::mordekaiser::Mordekaiser;

/// Q 延迟秒数（ron 未提供 castFrame，取权杖挥砸前摇近似值）
const MORDEKAISER_Q_DELAY: f32 = 0.3;

pub fn on_mordekaiser_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_morde.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });

    let start_distance =
        get_skill_data_value(spell_obj, "MaceStartDistance", skill.level).unwrap_or(400.0);
    let mace_length = get_skill_data_value(spell_obj, "MaceLength", skill.level).unwrap_or(625.0);
    let rect_width =
        get_skill_data_value(spell_obj, "RectangleWidth", skill.level).unwrap_or(160.0);

    commands.trigger(ActionDelayedDamage {
        entity,
        skill: skill.spell.clone(),
        skill_level: skill.level,
        delay: MORDEKAISER_Q_DELAY,
        point: trigger.point,
        origin: AoEOrigin::Caster,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Rectangle {
                width: rect_width,
                length: mace_length,
                start_distance,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "q_damage".to_string(),
                damage_type: DamageType::Magic,
                modifier: DamageModifier::Isolation {
                    scalar_data_value: "IsolationScalar".to_string(),
                },
            }],
            ..Default::default()
        }],
        indicator: AoEIndicator {
            color: Color::srgba(0.3, 1.0, 0.3, 0.35),
            pulse: false,
            grow_from_zero: true,
            impact_burst_scale: 1.3,
            fade_duration: 0.3,
        },
    });

    debug!(
        "莫德凯撒 Q 破灭之锤：矩形 [{},{}]×[-{},{}]，延迟 {}s，孤立增伤",
        start_distance,
        start_distance + mace_length,
        rect_width / 2.0,
        rect_width / 2.0,
        MORDEKAISER_Q_DELAY
    );
}