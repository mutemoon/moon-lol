pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::ekko::buffs::BuffEkkoPassive;

#[derive(Default)]
pub struct PluginEkko;

impl Plugin for PluginEkko {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ekko_q);
        app.add_observer(on_ekko_w);
        app.add_observer(on_ekko_e);
        app.add_observer(on_ekko_r);
        app.add_observer(on_ekko_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ekko"))]
#[reflect(Component)]
pub struct Ekko;

fn on_ekko_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ekko: Query<(), With<Ekko>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ekko.get(entity).is_err() {
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
    // Q is a projectile that slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1100.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_ekko_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ekko: Query<(), With<Ekko>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ekko.get(entity).is_err() {
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
    // W creates a zone that slows
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

fn on_ekko_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ekko: Query<(), With<Ekko>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ekko.get(entity).is_err() {
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
    // E is a dash that enhances next attack
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 325.0 },
        speed: 1000.0,
    });
}

fn on_ekko_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ekko: Query<(), With<Ekko>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ekko.get(entity).is_err() {
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
    // R is an AoE damage around previous position
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 500.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_ekko_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ekko: Query<(), With<Ekko>>,
) {
    let source = trigger.source;
    if q_ekko.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive stacks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffEkkoPassive::new());
    // Q and W slow
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 2.0));
}
