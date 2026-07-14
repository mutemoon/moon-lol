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

use crate::sivir::buffs::BuffSivirW;

#[derive(Default)]
pub struct PluginSivir;

impl Plugin for PluginSivir {
    fn build(&self, app: &mut App) {
        app.add_observer(on_sivir_q);
        app.add_observer(on_sivir_w);
        app.add_observer(on_sivir_e);
        app.add_observer(on_sivir_r);
        app.add_observer(on_sivir_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Sivir"))]
#[reflect(Component)]
pub struct Sivir;

fn on_sivir_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sivir: Query<(), With<Sivir>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sivir.get(entity).is_err() {
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
    // Q is boomerang blade - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 850.0,
                angle: 30.0,
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

fn on_sivir_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sivir: Query<(), With<Sivir>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sivir.get(entity).is_err() {
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
    // W is ricochet - attackspeed buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSivirW::new(0.5, 5.0));
}

fn on_sivir_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sivir: Query<(), With<Sivir>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sivir.get(entity).is_err() {
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
    // E is spell shield - magic shield;
}

fn on_sivir_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sivir: Query<(), With<Sivir>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sivir.get(entity).is_err() {
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
    // R is on the hunt - movespeed buff;
}

fn on_sivir_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sivir: Query<(), With<Sivir>>,
) {
    let source = trigger.source;
    if q_sivir.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // W gives attackspeed to caster
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffSivirW::new(0.5, 5.0));
}
