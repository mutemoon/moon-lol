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

use crate::draven::buffs::BuffDravenPassive;

#[derive(Default)]
pub struct PluginDraven;

impl Plugin for PluginDraven {
    fn build(&self, app: &mut App) {
        app.add_observer(on_draven_q);
        app.add_observer(on_draven_w);
        app.add_observer(on_draven_e);
        app.add_observer(on_draven_r);
        app.add_observer(on_draven_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Draven"))]
#[reflect(Component)]
pub struct Draven;

fn on_draven_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_draven: Query<(), With<Draven>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_draven.get(entity).is_err() {
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
    // Q enhances next attack - handled by buff system
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffDravenPassive::new());
}

fn on_draven_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_draven: Query<(), With<Draven>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_draven.get(entity).is_err() {
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
    // W is movement speed buff - handled by buff system;
}

fn on_draven_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_draven: Query<(), With<Draven>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_draven.get(entity).is_err() {
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
    // E is a knockback skillshot
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1100.0,
                angle: 45.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_draven_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_draven: Query<(), With<Draven>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_draven.get(entity).is_err() {
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
    // R is global damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 20000.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_draven_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_draven: Query<(), With<Draven>>,
) {
    let source = trigger.source;
    if q_draven.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.35, 2.0));
}
