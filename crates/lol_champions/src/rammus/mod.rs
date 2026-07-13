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

use crate::rammus::buffs::{BuffRammusE, BuffRammusQ, BuffRammusR};

#[derive(Default)]
pub struct PluginRammus;

impl Plugin for PluginRammus {
    fn build(&self, app: &mut App) {
        app.add_observer(on_rammus_q);
        app.add_observer(on_rammus_w);
        app.add_observer(on_rammus_e);
        app.add_observer(on_rammus_r);
        app.add_observer(on_rammus_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Rammus"))]
#[reflect(Component)]
pub struct Rammus;

fn on_rammus_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rammus: Query<(), With<Rammus>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rammus.get(entity).is_err() {
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
    // Q is powerball - damage and knockup
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 250.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_rammus_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rammus: Query<(), With<Rammus>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rammus.get(entity).is_err() {
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
    // W is defensive ball curl - damage reflection;
}

fn on_rammus_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rammus: Query<(), With<Rammus>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rammus.get(entity).is_err() {
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
    // E is frencying taunt - taunt enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 325.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_rammus_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rammus: Query<(), With<Rammus>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rammus.get(entity).is_err() {
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
    // R is soaring slam - AoE damage and slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 800.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_rammus_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_rammus: Query<(), With<Rammus>>,
) {
    let source = trigger.source;
    if q_rammus.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRammusQ::new(0.8, 1.0));
    // E taunts
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRammusE::new(2.0, 2.5));
    // R slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRammusR::new(0.5, 1.5));
}
