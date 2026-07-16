//! Renekton R - 终极统治 (Dominus)
//!
//! 变身，获得额外生命值，每秒产生怒气，对周围造成 AoE 伤害。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL4;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::DamageType;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::renekton::Renekton;
use crate::renekton::buffs::BuffRenektonR;

pub fn on_renekton_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_renekton: Query<(), With<Renekton>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_renekton.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 300.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffRenektonR::new(0.0, 5.0, 15.0));
}