pub mod e;
pub mod passive;
pub mod q;
pub mod r;
pub mod w;

#[cfg(test)]
mod e_tests;
#[cfg(test)]
mod passive_tests;
#[cfg(test)]
mod q_tests;
#[cfg(test)]
mod r_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod w_tests;

use bevy::prelude::*;
use lol_base::spell::Spell;
use lol_core::entities::champion::Champion;
use lol_core::life::Death;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_data_value};
use lol_core::team::Team;

#[derive(Default)]
pub struct PluginFiora;

impl Plugin for PluginFiora {
    fn build(&self, app: &mut App) {
        app.init_resource::<passive::FioraVitalLastDirection>();
        app.add_systems(
            FixedUpdate,
            (
                passive::attach_fiora_passive_ability,
                passive::update_add_vital,
                passive::update_remove_vital,
                passive::update_fiora_ms_buff,
                r::fixed_update,
                r::update_fiora_r_heal,
                e::update_fiora_e_buff,
                passive::update_vital_visuals,
                w::update_fiora_w,
            ),
        );
        app.add_observer(on_fiora_q);
        app.add_observer(on_fiora_w);
        app.add_observer(on_fiora_e);
        app.add_observer(on_fiora_r);
        app.add_observer(q::on_fiora_q_dash_end);
        app.add_observer(passive::on_passive_damage_create);
        app.add_observer(e::on_event_attack_end);
        app.add_observer(r::on_r_damage_create);
        app.add_observer(w::on_fiora_w_parried_cc);
    }
}

#[derive(Component, Default, Reflect)]
#[require(Champion, Name = Name::new("Fiora"))]
#[reflect(Component)]
pub struct Fiora;

fn on_fiora_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_fiora: Query<(), With<Fiora>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_fiora.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    q::cast_fiora_q(
        &mut commands,
        entity,
        trigger.point,
        skill.spell.clone(),
        skill.level,
        trigger.skill_entity,
    );
}

fn on_fiora_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_fiora: Query<(), With<Fiora>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_fiora.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    // W 参数来自 ron dataValues（BaseDamage / ParryDuration / CCDuration / MSSlowPercent）。
    let spell = res_spells.get(&skill.spell);
    let stab_damage = spell
        .and_then(|s| get_skill_data_value(s, "BaseDamage", skill.level))
        .unwrap_or(70.0);
    let _parry_duration = spell
        .and_then(|s| get_skill_data_value(s, "ParryDuration", skill.level))
        .unwrap_or(w::FIORA_W_PARRY_DURATION);
    let cc_duration = spell
        .and_then(|s| get_skill_data_value(s, "CCDuration", skill.level))
        .unwrap_or(w::FIORA_W_SLOW_DURATION.max(2.0));
    let slow_percent = spell
        .and_then(|s| get_skill_data_value(s, "MSSlowPercent", skill.level))
        .map(|v| v.abs())
        .unwrap_or(0.5);

    w::cast_fiora_w(
        &mut commands,
        entity,
        trigger.point,
        stab_damage,
        slow_percent,
        cc_duration,
    );
}

fn on_fiora_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_fiora: Query<(), With<Fiora>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_fiora.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    // E 攻速比例与第二击暴击比例来自 ron dataValues（ASPercent / AttackTwoPercentTAD）。
    let as_percent = res_spells
        .get(&skill.spell)
        .and_then(|s| get_skill_data_value(s, "ASPercent", skill.level))
        .unwrap_or(0.4);
    let crit_ratio = res_spells
        .get(&skill.spell)
        .and_then(|s| get_skill_data_value(s, "AttackTwoPercentTAD", skill.level))
        .map(|v| (v - 1.0).max(0.0))
        .unwrap_or(0.5);
    e::cast_fiora_e(&mut commands, entity, as_percent, crit_ratio);
}

fn on_fiora_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_fiora: Query<(), With<Fiora>>,
    q_skill: Query<&Skill>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_targets: Query<(Entity, &Transform, &Team), (With<Champion>, Without<Death>)>,
) {
    let entity = trigger.event_target();
    if q_fiora.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    r::cast_fiora_r(
        &mut commands,
        entity,
        trigger.point,
        &q_transform,
        &q_team,
        &q_targets,
        skill.level,
    );
}
