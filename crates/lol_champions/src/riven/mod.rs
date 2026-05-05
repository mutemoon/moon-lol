pub mod buffs;
pub mod e;
pub mod passive;
pub mod q;
pub mod r;
pub mod w;

#[cfg(test)]
mod e_tests;
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
use lol_core::attack::Attack;
use lol_core::base::buff::BuffOf;
use lol_core::damage::Damage;
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, get_skill_value,
};
use lol_core::team::Team;

use crate::riven::buffs::{BuffRivenR, BuffStun};

const RIVEN_R_DURATION: f32 = 15.0;

#[derive(Default)]
pub struct PluginRiven;

impl Plugin for PluginRiven {
    fn build(&self, app: &mut App) {
        app.add_observer(on_riven_skill_cast);
        app.add_observer(passive::on_damage_create_trigger_bonus);
        app.add_observer(q::on_riven_dash_end);
        app.add_systems(FixedUpdate, r::update_riven_buffs);
        app.add_systems(FixedUpdate, w::update_riven_stun);
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

fn on_riven_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_riven: Query<(), With<Riven>>,
    q_transform: Query<&Transform>,
    mut q_skill: Query<(&Skill, &mut CoolDown, Option<&SkillRecastWindow>)>,
    mut q_damage: Query<&mut Damage>,
    mut q_attack: Query<&mut Attack>,
    q_stun: Query<&BuffStun>,
    q_targets: Query<(Entity, &Team, &Transform, &Health)>,
    q_team: Query<&Team>,
    res_spells: Res<Assets<Spell>>,
    res_asset_server: Res<AssetServer>,
) {
    let entity = trigger.event_target();
    if q_riven.get(entity).is_err() {
        return;
    }

    // 眩晕中无法施放技能
    if q_stun.get(entity).is_ok() {
        return;
    }

    let Ok((skill, mut cooldown, recast)) = q_skill.get_mut(trigger.skill_entity) else {
        return;
    };

    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    // 预读取伤害值（后面可能修改）
    let damage_value = q_damage.get(entity).map(|d| d.0).unwrap_or(64.0);

    match skill.slot {
        SkillSlot::Q => q::cast_riven_q(
            &mut commands,
            entity,
            trigger.skill_entity,
            trigger.point,
            recast,
            skill.spell.clone(),
        ),
        SkillSlot::W => {
            w::cast_riven_w(&mut commands, entity, skill.spell.clone());

            // 对范围内敌人施加眩晕
            w::apply_w_stun_to_targets(&mut commands, entity, &q_transform, &q_team, &q_targets);
        }
        SkillSlot::E => {
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
        SkillSlot::R => {
            let is_recast = recast
                .as_ref()
                .map(|w| !w.timer.is_finished())
                .unwrap_or(false);

            if is_recast {
                // Wind Slash
                let missile_handles = [
                    res_asset_server.load("characters/riven/spells/RivenWindslashMissileRight.ron"),
                    res_asset_server
                        .load("characters/riven/spells/RivenWindslashMissileCenter.ron"),
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
            } else {
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
                    .insert(SkillRecastWindow::new(1, 1, RIVEN_R_DURATION));
            }
        }
        _ => {}
    }
}
