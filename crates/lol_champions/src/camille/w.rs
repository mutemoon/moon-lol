//! Camille W（战术扫荡 / Tactical Sweep）。
//!
//! 蓄力后朝扇形造成物理伤害，命中敌人时减速。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{ActionDamageEffect, DamageShape, TargetDamage, TargetFilter};
use lol_core::action::delayed_damage::{ActionDelayedDamage, AoEIndicator, AoEOrigin};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_data_value};

use crate::camille::Camille;

pub fn on_camille_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    let radius = get_skill_data_value(spell_obj, "BlastLength", skill.level).unwrap_or(650.0);
    let angle = get_skill_data_value(spell_obj, "ConeAngle", skill.level).unwrap_or(35.0);
    let delay = get_skill_data_value(spell_obj, "ChargeDuration", skill.level).unwrap_or(0.75);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    commands.trigger(ActionDelayedDamage {
        entity,
        skill: skill.spell.clone(),
        skill_level: skill.level,
        delay,
        point: trigger.point,
        origin: AoEOrigin::Caster,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector { radius, angle },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "base_damage_total".to_string(),
                damage_type: lol_core::damage::DamageType::Physical,
                ..Default::default()
            }],
            ..Default::default()
        }],
        indicator: AoEIndicator {
            color: Color::srgba(0.9, 0.3, 0.5, 0.4),
            pulse: true,
            grow_from_zero: false,
            impact_burst_scale: 1.4,
            fade_duration: 0.3,
        },
    });
}