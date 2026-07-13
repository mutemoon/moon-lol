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

use crate::shaco::buffs::BuffShacoW;

#[derive(Default)]
pub struct PluginShaco;

impl Plugin for PluginShaco {
    fn build(&self, app: &mut App) {
        app.add_observer(on_shaco_q);
        app.add_observer(on_shaco_w);
        app.add_observer(on_shaco_e);
        app.add_observer(on_shaco_r);
        app.add_observer(on_shaco_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Shaco"))]
#[reflect(Component)]
pub struct Shaco;

fn on_shaco_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shaco: Query<(), With<Shaco>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_shaco.get(entity).is_err() {
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
    // Q is vanish - invisibility;
}

fn on_shaco_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shaco: Query<(), With<Shaco>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_shaco.get(entity).is_err() {
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
    // W is jack inp - fear
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

fn on_shaco_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shaco: Query<(), With<Shaco>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_shaco.get(entity).is_err() {
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
    // E is two shiv - damage and slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 625.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_shaco_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shaco: Query<(), With<Shaco>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_shaco.get(entity).is_err() {
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
    // R is halluate - explosion;
}

fn on_shaco_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_shaco: Query<(), With<Shaco>>,
) {
    let source = trigger.source;
    if q_shaco.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W fears
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffShacoW::new(0.5, 1.0));
}
