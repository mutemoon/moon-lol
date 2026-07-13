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

use crate::braum::buffs::{BuffBraumPassive, BuffBraumW};

#[derive(Default)]
pub struct PluginBraum;

impl Plugin for PluginBraum {
    fn build(&self, app: &mut App) {
        app.add_observer(on_braum_q);
        app.add_observer(on_braum_w);
        app.add_observer(on_braum_e);
        app.add_observer(on_braum_r);
        app.add_observer(on_braum_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Braum"))]
#[reflect(Component)]
pub struct Braum;

fn on_braum_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_braum: Query<(), With<Braum>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_braum.get(entity).is_err() {
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
    // Q is a skillshot that slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1050.0,
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

fn on_braum_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_braum: Query<(), With<Braum>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_braum.get(entity).is_err() {
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
    // W jumps to ally and grants armor/mr buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffBraumW::new());
}

fn on_braum_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_braum: Query<(), With<Braum>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_braum.get(entity).is_err() {
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
    // E blocks projectiles - no direct damage;
}

fn on_braum_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_braum: Query<(), With<Braum>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_braum.get(entity).is_err() {
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
    // R is a knockup
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1200.0,
                angle: 45.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_braum_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_braum: Query<(), With<Braum>>,
) {
    let source = trigger.source;
    if q_braum.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.7, 2.0));
    // Apply passive stacks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBraumPassive::new());
}
