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

use crate::rakan::buffs::{BuffRakanR, BuffRakanW};

#[derive(Default)]
pub struct PluginRakan;

impl Plugin for PluginRakan {
    fn build(&self, app: &mut App) {
        app.add_observer(on_rakan_q);
        app.add_observer(on_rakan_w);
        app.add_observer(on_rakan_e);
        app.add_observer(on_rakan_r);
        app.add_observer(on_rakan_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Rakan"))]
#[reflect(Component)]
pub struct Rakan;

fn on_rakan_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rakan: Query<(), With<Rakan>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rakan.get(entity).is_err() {
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
    // Q is gleaming quill - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 900.0,
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

fn on_rakan_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rakan: Query<(), With<Rakan>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rakan.get(entity).is_err() {
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
    // W is grand entrance - knockup
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 650.0 },
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

fn on_rakan_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rakan: Query<(), With<Rakan>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rakan.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let _skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    // E is battle dance - shield to ally;
}

fn on_rakan_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rakan: Query<(), With<Rakan>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rakan.get(entity).is_err() {
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
    // R is the quickness - damage and charm
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 150.0 },
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

fn on_rakan_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_rakan: Query<(), With<Rakan>>,
) {
    let source = trigger.source;
    if q_rakan.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W knockup
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRakanW::new(1.0, 1.5));
    // R charm and slow
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRakanR::new(1.5, 0.75, 2.0));
}
