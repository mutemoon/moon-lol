pub mod buffs;
pub mod e;
pub mod passive;
pub mod q;
pub mod r;
pub mod w;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

#[derive(Default)]
pub struct PluginMordekaiser;

impl Plugin for PluginMordekaiser {
    fn build(&self, app: &mut App) {
        app.add_observer(on_mordekaiser_q);
        app.add_observer(on_mordekaiser_w);
        app.add_observer(on_mordekaiser_e);
        app.add_observer(on_mordekaiser_r);
    }
}

#[derive(Component, Default, Reflect)]
#[require(Champion, Name = Name::new("Mordekaiser"))]
#[reflect(Component)]
pub struct Mordekaiser;

fn on_mordekaiser_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_morde.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    q::cast_mordekaiser_q(&mut commands, entity, trigger.point);
}

fn on_mordekaiser_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_morde.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    w::cast_mordekaiser_w(&mut commands, entity);
}

fn on_mordekaiser_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_morde.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    e::cast_mordekaiser_e(&mut commands, entity, trigger.point);
}

fn on_mordekaiser_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_morde.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    r::cast_mordekaiser_r(&mut commands, entity, trigger.point);
}
