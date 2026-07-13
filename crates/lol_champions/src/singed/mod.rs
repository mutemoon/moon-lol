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

use crate::singed::buffs::BuffSingedE;

#[derive(Default)]
pub struct PluginSinged;

impl Plugin for PluginSinged {
    fn build(&self, app: &mut App) {
        app.add_observer(on_singed_q);
        app.add_observer(on_singed_w);
        app.add_observer(on_singed_e);
        app.add_observer(on_singed_r);
        app.add_observer(on_singed_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Singed"))]
#[reflect(Component)]
pub struct Singed;

fn on_singed_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_singed: Query<(), With<Singed>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_singed.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q is poison trail - damage over time;
}

fn on_singed_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_singed: Query<(), With<Singed>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_singed.get(entity).is_err() {
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
    // W is mega adhesive - slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 400.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_singed_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_singed: Query<(), With<Singed>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_singed.get(entity).is_err() {
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
    // E is fling - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 400.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_singed_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_singed: Query<(), With<Singed>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_singed.get(entity).is_err() {
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
    // R is insanity - movespeed buff;
}

fn on_singed_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_singed: Query<(), With<Singed>>,
) {
    let source = trigger.source;
    if q_singed.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSingedE::new(0.6, 3.0));
}
