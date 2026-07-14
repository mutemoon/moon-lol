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

use crate::rengar::buffs::{BuffRengarE, BuffRengarR};

#[derive(Default)]
pub struct PluginRengar;

impl Plugin for PluginRengar {
    fn build(&self, app: &mut App) {
        app.add_observer(on_rengar_q);
        app.add_observer(on_rengar_w);
        app.add_observer(on_rengar_e);
        app.add_observer(on_rengar_r);
        app.add_observer(on_rengar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Rengar"))]
#[reflect(Component)]
pub struct Rengar;

fn on_rengar_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rengar: Query<(), With<Rengar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rengar.get(entity).is_err() {
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
    // Q is savagery - enhanced attack;
}

fn on_rengar_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rengar: Query<(), With<Rengar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rengar.get(entity).is_err() {
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
    // W is battle roar - AoE damage and heal
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 500.0 },
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

fn on_rengar_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rengar: Query<(), With<Rengar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rengar.get(entity).is_err() {
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
    // E is bola strike - damage and slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1000.0,
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

fn on_rengar_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rengar: Query<(), With<Rengar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rengar.get(entity).is_err() {
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
    // R is thrill of the hunt - camouflage and movespeed;
}

fn on_rengar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_rengar: Query<(), With<Rengar>>,
) {
    let source = trigger.source;
    if q_rengar.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRengarE::new(0.4, 2.25));
    // R gives movespeed
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRengarR::new(0.5, 14.0));
}
