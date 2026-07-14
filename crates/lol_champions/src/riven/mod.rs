pub mod buffs;
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
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::attack::Attack;
use lol_core::base::buff::BuffOf;
use lol_core::damage::Damage;
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, get_skill_value,
};
use lol_core::team::Team;

use crate::riven::buffs::BuffRivenR;

const RIVEN_R_DURATION: f32 = 15.0;

#[derive(Default)]
pub struct PluginRiven;

impl Plugin for PluginRiven {
    fn build(&self, app: &mut App) {
        app.add_observer(on_riven_q);
        app.add_observer(on_riven_w);
        app.add_observer(on_riven_e);
        app.add_observer(on_riven_r);
        app.add_observer(passive::on_riven_skill_cast_charge_passive);
        app.add_observer(passive::on_damage_create_trigger_bonus);
        app.add_observer(q::on_riven_dash_end);
        app.add_systems(FixedUpdate, r::update_riven_buffs);
        app.add_systems(FixedUpdate, passive::update_riven_passive_timer);
        app.add_systems(FixedUpdate, buffs::update_shield_visuals);
        app.add_systems(FixedUpdate, buffs::cleanup_shield_visuals);
    }
}

#[derive(Component, Default, Reflect)]
#[require(Champion, Name = Name::new("Riven"))]
#[reflect(Component)]
pub struct Riven;

fn get_r_cooldown(level: usize) -> f32 {
    match level {
        1 => 120.0,
        2 => 90.0,
        3 => 60.0,
        _ => 120.0,
    }
}

fn on_riven_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_riven: Query<(), With<Riven>>,
    q_skill: Query<(&Skill, Option<&SkillRecastWindow>)>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_riven.get(entity).is_err() {
        return;
    }

    let Ok((skill, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    let damage_value = q_damage.get(entity).map(|d| d.0).unwrap_or(64.0);

    let q_damage = get_skill_value(spell_obj, "first_slash_damage", skill.level, |stat| {
        if stat == 2 { damage_value } else { 0.0 }
    })
    .unwrap_or(0.0);
    q::cast_riven_q(
        &mut commands,
        entity,
        trigger.skill_entity,
        trigger.point,
        recast,
        q_damage,
    );
}

fn on_riven_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_riven: Query<(), With<Riven>>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_targets: Query<(Entity, &Team, &Transform, &Health)>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_riven.get(entity).is_err() {
        return;
    }

    // 眩晕/被控由统一施法管线（CastBlock）拦截，此处无需重复检查
    let _ = entity;

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    let damage_value = q_damage.get(entity).map(|d| d.0).unwrap_or(64.0);

    let w_damage = get_skill_value(spell_obj, "total_damage", skill.level, |stat| {
        if stat == 2 { damage_value } else { 0.0 }
    })
    .unwrap_or(150.0);

    w::cast_riven_w(&mut commands, entity, w_damage);

    // 对范围内敌人施加眩晕
    w::apply_w_stun_to_targets(&mut commands, entity, &q_transform, &q_team, &q_targets);
}

fn on_riven_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_riven: Query<(), With<Riven>>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
    q_transform: Query<&Transform>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_riven.get(entity).is_err() {
        return;
    }

    // 眩晕/被控由统一施法管线（CastBlock）拦截，此处无需重复检查
    let _ = entity;

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    let damage_value = q_damage.get(entity).map(|d| d.0).unwrap_or(64.0);

    let shield_value = get_skill_value(spell_obj, "total_shield", skill.level, |stat| {
        if stat == 2 { damage_value } else { 0.0 }
    })
    .unwrap_or(100.0);

    e::cast_riven_e(
        &mut commands,
        &q_transform,
        entity,
        trigger.point,
        shield_value,
    );
}

fn on_riven_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_riven: Query<(), With<Riven>>,
    mut q_skill: Query<(&Skill, &mut CoolDown, Option<&SkillRecastWindow>)>,
    mut q_damage: Query<&mut Damage>,
    mut q_attack: Query<&mut Attack>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_targets: Query<(Entity, &Team, &Transform, &Health)>,
    res_spells: Res<Assets<Spell>>,
    res_asset_server: Res<AssetServer>,
) {
    let entity = trigger.event_target();
    if q_riven.get(entity).is_err() {
        return;
    }

    // 眩晕/被控由统一施法管线（CastBlock）拦截，此处无需重复检查
    let _ = entity;

    let Ok((skill, mut cooldown, recast)) = q_skill.get_mut(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    let damage_value = q_damage.get(entity).map(|d| d.0).unwrap_or(64.0);

    let stage = recast.map(|w| w.stage).unwrap_or(1);

    match stage {
        2 => {
            // Wind Slash
            let missile_handles = [
                res_asset_server.load("characters/riven/spells/RivenWindslashMissileRight.ron"),
                res_asset_server.load("characters/riven/spells/RivenWindslashMissileCenter.ron"),
                res_asset_server.load("characters/riven/spells/RivenWindslashMissileLeft.ron"),
            ];
            r::cast_riven_wind_slash(
                &mut commands,
                entity,
                &missile_handles,
                &q_transform,
                &q_team,
                &q_targets,
                spell_obj,
                skill.level,
                damage_value,
            );

            commands
                .entity(trigger.skill_entity)
                .remove::<SkillRecastWindow>();

            let r_cd = get_r_cooldown(skill.level);
            cooldown.duration = r_cd;
            cooldown.timer = Some(Timer::from_seconds(r_cd, TimerMode::Once));
        }
        _ => {
            // 初次 R - 获取增伤、开启连招窗口
            let bonus_ad = damage_value * 0.25;
            let bonus_range = 75.0;

            if let Ok(mut dmg) = q_damage.get_mut(entity) {
                dmg.0 += bonus_ad;
            }
            if let Ok(mut atk) = q_attack.get_mut(entity) {
                atk.range += bonus_range;
            }

            commands.entity(entity).with_related::<BuffOf>(BuffRivenR {
                timer: Timer::from_seconds(RIVEN_R_DURATION, TimerMode::Once),
            });

            // 覆盖冷却为真实 R 冷却，同时添加连招窗口
            let r_cd = get_r_cooldown(skill.level);
            cooldown.duration = r_cd;
            cooldown.timer = Some(Timer::from_seconds(r_cd, TimerMode::Once));

            commands
                .entity(trigger.skill_entity)
                .insert(SkillRecastWindow::new(2, 2, RIVEN_R_DURATION));

            commands.trigger(CommandAnimationPlay {
                entity,
                hash: "Spell4A".to_string(),
                repeat: false,
                duration: None,
            });
        }
    }
}
