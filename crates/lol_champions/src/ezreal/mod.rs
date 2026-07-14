pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::ezreal::buffs::BuffEzrealPassive;

#[derive(Default)]
pub struct PluginEzreal;

impl Plugin for PluginEzreal {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ezreal_q);
        app.add_observer(on_ezreal_w);
        app.add_observer(on_ezreal_e);
        app.add_observer(on_ezreal_r);
        app.add_observer(on_ezreal_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ezreal"))]
#[reflect(Component)]
pub struct Ezreal;

fn on_ezreal_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ezreal: Query<(), With<Ezreal>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ezreal.get(entity).is_err() {
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
    // Q is a long range skillshot
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1200.0,
                angle: 15.0,
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

fn on_ezreal_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ezreal: Query<(), With<Ezreal>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ezreal.get(entity).is_err() {
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
    // W marks target
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1200.0,
                angle: 20.0,
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

fn on_ezreal_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ezreal: Query<(), With<Ezreal>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ezreal.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let point = trigger.point;
    let _skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    // E is a blink/dash
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 475.0 },
        speed: 800.0,
    });
}

fn on_ezreal_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ezreal: Query<(), With<Ezreal>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ezreal.get(entity).is_err() {
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
    // R is global AoE
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 20000.0,
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

fn on_ezreal_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ezreal: Query<(), With<Ezreal>>,
) {
    let source = trigger.source;
    if q_ezreal.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive stacks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffEzrealPassive::new());
}
