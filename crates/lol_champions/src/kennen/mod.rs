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

use crate::kennen::buffs::{BuffKennenE, BuffKennenMarkOfStorm, BuffKennenR};

#[derive(Default)]
pub struct PluginKennen;

impl Plugin for PluginKennen {
    fn build(&self, app: &mut App) {
        app.add_observer(on_kennen_q);
        app.add_observer(on_kennen_w);
        app.add_observer(on_kennen_e);
        app.add_observer(on_kennen_r);
        app.add_observer(on_kennen_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kennen"))]
#[reflect(Component)]
pub struct Kennen;

fn on_kennen_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kennen: Query<(), With<Kennen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kennen.get(entity).is_err() {
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

    // Q is a shuriken that applies mark
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1050.0,
                angle: 10.0,
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

fn on_kennen_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kennen: Query<(), With<Kennen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kennen.get(entity).is_err() {
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

    // W deals damage to marked enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 775.0 },
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

fn on_kennen_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kennen: Query<(), With<Kennen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kennen.get(entity).is_err() {
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

    // E grants movespeed and immunity
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKennenE::new(1.0, 0.6, 2.0));
}

fn on_kennen_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kennen: Query<(), With<Kennen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kennen.get(entity).is_err() {
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

    // R summons storm that damages and applies marks
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 550.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });

    // R grants armor/mr
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKennenR::new(40.0, 40.0, 3.0));
}

fn on_kennen_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kennen: Query<(), With<Kennen>>,
) {
    let source = trigger.source;
    if q_kennen.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply mark of the storm (3 marks = stun)
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffKennenMarkOfStorm::new(1, 8.0));
}
