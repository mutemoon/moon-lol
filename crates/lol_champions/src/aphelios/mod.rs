pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::aphelios::buffs::{BuffApheliosCalibrum, BuffApheliosGravitum};

#[derive(Default)]
pub struct PluginAphelios;

impl Plugin for PluginAphelios {
    fn build(&self, app: &mut App) {
        app.add_observer(on_aphelios_q);
        app.add_observer(on_aphelios_r);
        app.add_observer(on_aphelios_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Aphelios"))]
#[reflect(Component)]
pub struct Aphelios;

fn on_aphelios_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aphelios: Query<(), With<Aphelios>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_aphelios.get(entity).is_err() {
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
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1450.0,
                angle: 25.0,
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

fn on_aphelios_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aphelios: Query<(), With<Aphelios>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_aphelios.get(entity).is_err() {
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
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 1300.0 },
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

fn on_aphelios_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_aphelios: Query<(), With<Aphelios>>,
) {
    let source = trigger.source;
    if q_aphelios.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffApheliosCalibrum::new(70.0, 2.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffApheliosGravitum::new(0.5, 2.0));
}
