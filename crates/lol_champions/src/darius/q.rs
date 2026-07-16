//! Darius Q - 大杀四方 (Decimate)
//!
//! Inner blade (handle): lower damage, does NOT stack Hemorrhage
//! Outer blade (axe): higher damage, stacks Hemorrhage
//!
//! Inner radius: ~150
//! Outer radius: ~350

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL1;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{ActionDamageEffect, DamageShape, TargetDamage, TargetFilter};
use lol_core::action::delayed_damage::{ActionDelayedDamage, AoEIndicator, AoEOrigin};
use lol_core::damage::DamageType;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, delay_from_cast_frame};

use crate::darius::buffs::DARIUS_Q_INNER_TAG;
use crate::darius::Darius;

/// Inner blade radius (the "handle" of the axe)
pub const DARIUS_Q_INNER_RADIUS: f32 = 150.0;

/// Outer blade radius (the "blade" of the axe)
pub const DARIUS_Q_OUTER_RADIUS: f32 = 350.0;

pub fn on_darius_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_darius.get(entity).is_err() {
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

    let delay = delay_from_cast_frame(spell_obj);

    // 内圈（Circle）+ 外圈（Annular）双形，均用 blade_damage 物理伤害
    let inner_effect = ActionDamageEffect {
        shape: DamageShape::Circle {
            radius: DARIUS_Q_INNER_RADIUS,
        },
        damage_list: vec![TargetDamage {
            filter: TargetFilter::All,
            amount: "blade_damage".to_string(),
            damage_type: DamageType::Physical,
            ..Default::default()
        }],
        tag: Some(DARIUS_Q_INNER_TAG),
        ..Default::default()
    };
    let outer_effect = ActionDamageEffect {
        shape: DamageShape::Annular {
            inner_radius: DARIUS_Q_INNER_RADIUS,
            outer_radius: DARIUS_Q_OUTER_RADIUS,
        },
        damage_list: vec![TargetDamage {
            filter: TargetFilter::All,
            amount: "blade_damage".to_string(),
            damage_type: DamageType::Physical,
            ..Default::default()
        }],
        ..Default::default()
    };

    commands.trigger(ActionDelayedDamage {
        entity,
        skill: skill.spell.clone(),
        skill_level: skill.level,
        delay,
        point: trigger.point,
        origin: AoEOrigin::Caster,
        effects: vec![inner_effect, outer_effect],
        indicator: AoEIndicator {
            color: Color::srgba(0.9, 0.2, 0.2, 0.4),
            pulse: false,
            grow_from_zero: true,
            impact_burst_scale: 1.4,
            fade_duration: 0.3,
        },
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_darius_q_inner_radius() {
        assert!(DARIUS_Q_INNER_RADIUS > 0.0);
        assert!(DARIUS_Q_OUTER_RADIUS > DARIUS_Q_INNER_RADIUS);
    }
}