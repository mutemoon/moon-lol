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

use crate::qiyana::buffs::BuffQiyanaW;

#[derive(Default)]
pub struct PluginQiyana;

impl Plugin for PluginQiyana {
    fn build(&self, app: &mut App) {
        app.add_observer(on_qiyana_q);
        app.add_observer(on_qiyana_w);
        app.add_observer(on_qiyana_e);
        app.add_observer(on_qiyana_r);
        app.add_observer(on_qiyana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Qiyana"))]
#[reflect(Component)]
pub struct Qiyana;

fn on_qiyana_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_qiyana: Query<(), With<Qiyana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_qiyana.get(entity).is_err() {
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
    // Q is edge of Ixtal - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 525.0,
                angle: 30.0,
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

fn on_qiyana_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_qiyana: Query<(), With<Qiyana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_qiyana.get(entity).is_err() {
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
    // W is elemental wrath - dash and element buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffQiyanaW::new(0.2, 5.0));
}

fn on_qiyana_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_qiyana: Query<(), With<Qiyana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_qiyana.get(entity).is_err() {
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
    // E is terrashape - dash through terrain;
}

fn on_qiyana_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_qiyana: Query<(), With<Qiyana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_qiyana.get(entity).is_err() {
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
    // R is audacity/supreme display - large AoE knockup
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 875.0 },
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

fn on_qiyana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_qiyana: Query<(), With<Qiyana>>,
) {
    let source = trigger.source;
    if q_qiyana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W grass gives movespeed
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffQiyanaW::new(0.2, 5.0));
}
