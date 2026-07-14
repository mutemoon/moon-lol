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

use crate::senna::buffs::BuffSennaW;

#[derive(Default)]
pub struct PluginSenna;

impl Plugin for PluginSenna {
    fn build(&self, app: &mut App) {
        app.add_observer(on_senna_q);
        app.add_observer(on_senna_w);
        app.add_observer(on_senna_e);
        app.add_observer(on_senna_r);
        app.add_observer(on_senna_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Senna"))]
#[reflect(Component)]
pub struct Senna;

fn on_senna_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_senna: Query<(), With<Senna>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_senna.get(entity).is_err() {
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
    // Q is duskblade of shadow - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 600.0,
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

fn on_senna_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_senna: Query<(), With<Senna>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_senna.get(entity).is_err() {
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
    // W is last embrace - root
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 1000.0 },
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

fn on_senna_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_senna: Query<(), With<Senna>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_senna.get(entity).is_err() {
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
    // E is curtain of darkness - camouflage;
}

fn on_senna_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_senna: Query<(), With<Senna>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_senna.get(entity).is_err() {
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
    // R is pierce the veil - AoE damage and shield
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 2500.0,
                angle: 50.0,
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

fn on_senna_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_senna: Query<(), With<Senna>>,
) {
    let source = trigger.source;
    if q_senna.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSennaW::new(1.5, 2.0));
}
