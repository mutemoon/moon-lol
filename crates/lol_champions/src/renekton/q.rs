//! Renekton Q - 暴君狂击 (Cleave)
//!
//! 对周围敌人造成物理伤害，50 怒气以上消耗怒气强化伤害和治疗。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL1;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::ability_resource::AbilityResource;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::common_buffs::BuffSelfHeal;
use lol_core::damage::DamageType;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::renekton::Renekton;

pub fn on_renekton_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_renekton: Query<(), With<Renekton>>,
    mut q_ability_resource: Query<&mut AbilityResource>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_renekton.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q is a cleave that deals damage in a circle
    let rage = q_ability_resource
        .get(entity)
        .map(|r| r.value)
        .unwrap_or(0.0);
    if rage >= 50.0 {
        // 消耗 50 怒气，强化版伤害和治疗
        if let Ok(mut resource) = q_ability_resource.get_mut(entity) {
            resource.value -= 50.0;
        }
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
            .with_related::<BuffOf>(BuffSelfHeal::new(80.0)); // 翻倍治疗
    } else {
        commands.trigger(ActionDamage {
            entity,
            skill: skill_spell,
            effects: vec![ActionDamageEffect {
                shape: DamageShape::Circle { radius: 250.0 },
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
            .with_related::<BuffOf>(BuffSelfHeal::new(40.0));
    };
}
