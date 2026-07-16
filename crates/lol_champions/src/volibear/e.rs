//! Volibear E - 落雷 (Sky Splitter)
//!
//! 地面靶向延迟 AoE 魔法伤害 + 落地护盾。减速由 on_volibear_damage_hit 施加。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{ActionDamageEffect, DamageShape, TargetDamage, TargetFilter};
use lol_core::action::delayed_damage::{ActionDelayedDamage, AoEIndicator, AoEOrigin};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::DamageType;
use lol_core::skill::{
    EventSkillCast, Skill, SkillSlot, delay_from_cast_frame, get_skill_cast_radius,
};

use crate::volibear::Volibear;

pub fn on_volibear_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_volibear.get(entity).is_err() {
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

    let radius = get_skill_cast_radius(spell_obj, skill.level).unwrap_or(325.0);

    commands.trigger(ActionDelayedDamage {
        entity,
        skill: skill.spell.clone(),
        skill_level: skill.level,
        delay: delay_from_cast_frame(spell_obj),
        point: trigger.point,
        origin: AoEOrigin::CastPoint,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "calculated_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
        indicator: AoEIndicator {
            color: Color::srgba(0.4, 0.6, 1.0, 0.4),
            pulse: false,
            grow_from_zero: true,
            impact_burst_scale: 1.5,
            fade_duration: 0.3,
        },
    });

    // 落地护盾
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffShieldWhite::new(100.0));
}