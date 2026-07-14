pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::samira::buffs::BuffSamiraE;

#[derive(Default)]
pub struct PluginSamira;

impl Plugin for PluginSamira {
    fn build(&self, app: &mut App) {
        app.add_observer(on_samira_q);
        app.add_observer(on_samira_w);
        app.add_observer(on_samira_e);
        app.add_observer(on_samira_r);
        app.add_observer(on_samira_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Samira"))]
#[reflect(Component)]
pub struct Samira;

fn on_samira_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_samira: Query<(), With<Samira>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_samira.get(entity).is_err() {
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
    // Q is flonen - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 600.0,
                angle: 25.0,
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

fn on_samira_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_samira: Query<(), With<Samira>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_samira.get(entity).is_err() {
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
    // W is blade storm - AoE damage
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
}

fn on_samira_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_samira: Query<(), With<Samira>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_samira.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    // E is blade rush - dash;
}

fn on_samira_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_samira: Query<(), With<Samira>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_samira.get(entity).is_err() {
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
    // R is infernum - large AoE damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 900.0,
                angle: 50.0,
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

fn on_samira_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_samira: Query<(), With<Samira>>,
) {
    let source = trigger.source;
    if q_samira.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSamiraE::new(0.75, 1.0));
}
