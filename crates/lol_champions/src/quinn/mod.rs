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

use crate::quinn::buffs::{BuffQuinnE, BuffQuinnW};

#[derive(Default)]
pub struct PluginQuinn;

impl Plugin for PluginQuinn {
    fn build(&self, app: &mut App) {
        app.add_observer(on_quinn_q);
        app.add_observer(on_quinn_w);
        app.add_observer(on_quinn_e);
        app.add_observer(on_quinn_r);
        app.add_observer(on_quinn_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Quinn"))]
#[reflect(Component)]
pub struct Quinn;

fn on_quinn_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_quinn: Query<(), With<Quinn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_quinn.get(entity).is_err() {
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
    // Q is blinding assault - damage and blind
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1050.0,
                angle: 20.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_quinn_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_quinn: Query<(), With<Quinn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_quinn.get(entity).is_err() {
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
    // W is heightened senses - attackspeed and movespeed buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffQuinnW::new(0.8, 0.4, 2.0));
}

fn on_quinn_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_quinn: Query<(), With<Quinn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_quinn.get(entity).is_err() {
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
    // E is vault - knockback and slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 600.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_quinn_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_quinn: Query<(), With<Quinn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_quinn.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R is behind enemy lines - high movespeed;
}

fn on_quinn_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_quinn: Query<(), With<Quinn>>,
) {
    let source = trigger.source;
    if q_quinn.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffQuinnE::new(0.5, 1.5));
}
