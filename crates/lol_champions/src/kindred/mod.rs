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

use crate::kindred::buffs::{BuffKindredE, BuffKindredW};

#[derive(Default)]
pub struct PluginKindred;

impl Plugin for PluginKindred {
    fn build(&self, app: &mut App) {
        app.add_observer(on_kindred_q);
        app.add_observer(on_kindred_w);
        app.add_observer(on_kindred_e);
        app.add_observer(on_kindred_r);
        app.add_observer(on_kindred_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kindred"))]
#[reflect(Component)]
pub struct Kindred;

fn on_kindred_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kindred: Query<(), With<Kindred>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kindred.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let _point = trigger.point;
    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });

    // Q is a dash that shoots arrows
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
}

fn on_kindred_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kindred: Query<(), With<Kindred>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kindred.get(entity).is_err() {
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

    // W marks an area where Wolf attacks
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKindredW::new(50.0, 8.5));

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 500.0 },
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

fn on_kindred_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kindred: Query<(), With<Kindred>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kindred.get(entity).is_err() {
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

    // E marks and slows, 3 marks = Wolf attacks
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 500.0,
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

fn on_kindred_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kindred: Query<(), With<Kindred>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kindred.get(entity).is_err() {
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

    // R creates a protective zone
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 535.0 },
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

fn on_kindred_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kindred: Query<(), With<Kindred>>,
) {
    let source = trigger.source;
    if q_kindred.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply slow and mark
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffKindredE::new(1, 0.3, 2.0));
}
