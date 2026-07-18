//! Garen E - 审判 (Judgment)
//!
//! 周围 AoE 旋转造成物理伤害。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::damage::DamageType;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::garen::Garen;

pub fn on_garen_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_garen: Query<(), With<Garen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_garen.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 200.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}
