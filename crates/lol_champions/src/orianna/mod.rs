pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::orianna::buffs::BuffOriannaE;

#[derive(Default)]
pub struct PluginOrianna;

impl Plugin for PluginOrianna {
    fn build(&self, app: &mut App) {
        app.add_observer(on_orianna_q);
        app.add_observer(on_orianna_w);
        app.add_observer(on_orianna_e);
        app.add_observer(on_orianna_r);
        app.add_observer(on_orianna_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Orianna"))]
#[reflect(Component)]
pub struct Orianna;

fn on_orianna_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_orianna: Query<(), With<Orianna>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_orianna.get(entity).is_err() {
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
    // Q commands ball to location
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 825.0 },
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

fn on_orianna_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_orianna: Query<(), With<Orianna>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_orianna.get(entity).is_err() {
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
    // W deals damage and slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 225.0 },
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

fn on_orianna_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_orianna: Query<(), With<Orianna>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_orianna.get(entity).is_err() {
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
    // E shields ball carrier
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOriannaE::new(100.0, 30.0, 4.0));

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 1120.0 },
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

fn on_orianna_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_orianna: Query<(), With<Orianna>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_orianna.get(entity).is_err() {
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
    // R is a shockwave that knocks back
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 415.0 },
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

fn on_orianna_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_orianna: Query<(), With<Orianna>>,
) {
    let source = trigger.source;
    if q_orianna.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 2.0));
}
