//! Darius R - 诺克萨斯断头台 (Noxian Guillotine)
//!
//! 对最近敌方英雄造成真实伤害，伤害随目标出血层数 +20%/层

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL4;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::base::buff::Buffs;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_data_value, get_skill_value};
use lol_core::team::Team;

use crate::darius::Darius;
use crate::darius::buffs::BuffDariusBleed;

/// R 斩杀范围（最近敌方英雄）。
const DARIUS_R_RANGE: f32 = 475.0;

pub fn on_darius_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
    q_buffs: Query<&Buffs>,
    q_bleed: Query<&BuffDariusBleed>,
    q_damage: Query<&Damage>,
) {
    let entity = trigger.event_target();
    if q_darius.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    // 解析 R 基础伤害公式（r_base_damage + 0.75*AD）
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let base = res_spells
        .get(&skill.spell)
        .and_then(|s| {
            get_skill_value(
                s,
                "damage",
                skill.level,
                |stat| if stat == 2 { ad } else { 0.0 },
            )
        })
        .unwrap_or(0.0);

    // 范围内最近敌方英雄
    let origin = q_transform
        .get(entity)
        .map(|t| t.translation.xz())
        .unwrap_or_default();
    let own_team = q_team.get(entity).ok();
    let mut nearest: Option<(Entity, f32)> = None;
    for (enemy, enemy_tf) in q_enemies.iter() {
        if enemy == entity || q_team.get(enemy).ok() == own_team {
            continue;
        }
        let d = enemy_tf.translation.xz().distance(origin);
        if d > DARIUS_R_RANGE {
            continue;
        }
        if nearest.map(|(_, nd)| d < nd).unwrap_or(true) {
            nearest = Some((enemy, d));
        }
    }

    // 目标出血层数 -> 每层 RDamagePercentPerHemoStack（spell data = 20%）
    let stacks = nearest
        .and_then(|(enemy, _)| q_buffs.get(enemy).ok())
        .map(|buffs| {
            buffs
                .iter()
                .filter_map(|b| q_bleed.get(b).ok())
                .next()
                .map(|bl| bl.stacks)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    let per_stack = res_spells
        .get(&skill.spell)
        .and_then(|s| get_skill_data_value(s, "RDamagePercentPerHemoStack", skill.level))
        .unwrap_or(0.2);
    let amount = base * (1.0 + per_stack * stacks as f32);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    if let Some(target) = nearest.map(|(e, _)| e) {
        commands.entity(target).trigger(|e| CommandDamageCreate {
            entity: e,
            source: entity,
            damage_type: DamageType::True,
            amount,
            tag: None,
        });
    }
}