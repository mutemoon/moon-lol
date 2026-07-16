//! Renekton W - 冷酷捕猎 (Ruthless Predator)
//!
//! 强化下次普攻，造成伤害并眩晕目标。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter};
use lol_core::attack::CommandAttackReset;
use lol_core::damage::DamageType;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::renekton::Renekton;

pub fn on_renekton_w(
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
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W is an empowered auto attack that stuns
    commands.trigger(CommandAttackReset { entity });
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 150.0,
            },
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