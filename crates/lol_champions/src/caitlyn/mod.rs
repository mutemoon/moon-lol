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

use crate::caitlyn::buffs::BuffCaitlynPassive;

#[derive(Default)]
pub struct PluginCaitlyn;

impl Plugin for PluginCaitlyn {
    fn build(&self, app: &mut App) {
        app.add_observer(on_caitlyn_q);
        app.add_observer(on_caitlyn_w);
        app.add_observer(on_caitlyn_e);
        app.add_observer(on_caitlyn_r);
        app.add_observer(on_caitlyn_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Caitlyn"))]
#[reflect(Component)]
pub struct Caitlyn;

fn on_caitlyn_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_caitlyn: Query<(), With<Caitlyn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_caitlyn.get(entity).is_err() {
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
    // Q is a long range piercing shot
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1300.0,
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

fn on_caitlyn_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_caitlyn: Query<(), With<Caitlyn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_caitlyn.get(entity).is_err() {
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
    // W places traps - no direct damage;
}

fn on_caitlyn_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_caitlyn: Query<(), With<Caitlyn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_caitlyn.get(entity).is_err() {
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
    // E is a net that slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 800.0,
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

fn on_caitlyn_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_caitlyn: Query<(), With<Caitlyn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_caitlyn.get(entity).is_err() {
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
    // R is a global targeted shot
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 3500.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::Champion,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_caitlyn_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_caitlyn: Query<(), With<Caitlyn>>,
) {
    let source = trigger.source;
    if q_caitlyn.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 1.0));
    // Apply headshot passive stacks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffCaitlynPassive::new());
}
