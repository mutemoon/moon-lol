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

use crate::skarner::buffs::BuffSkarnerR;

#[derive(Default)]
pub struct PluginSkarner;

impl Plugin for PluginSkarner {
    fn build(&self, app: &mut App) {
        app.add_observer(on_skarner_q);
        app.add_observer(on_skarner_w);
        app.add_observer(on_skarner_e);
        app.add_observer(on_skarner_r);
        app.add_observer(on_skarner_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Skarner"))]
#[reflect(Component)]
pub struct Skarner;

fn on_skarner_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_skarner: Query<(), With<Skarner>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_skarner.get(entity).is_err() {
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
    // Q is crystal slash - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 350.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_skarner_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_skarner: Query<(), With<Skarner>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_skarner.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W is crystalline exoskeleton - shield;
}

fn on_skarner_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_skarner: Query<(), With<Skarner>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_skarner.get(entity).is_err() {
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
    // E is fracture - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 600.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_skarner_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_skarner: Query<(), With<Skarner>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_skarner.get(entity).is_err() {
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
    // R is impale - stun and damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 350.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_skarner_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_skarner: Query<(), With<Skarner>>,
) {
    let source = trigger.source;
    if q_skarner.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // R stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSkarnerR::new(1.0, 1.5));
}
