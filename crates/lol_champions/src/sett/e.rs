//! Sett E - 迎面痛击 (Facebreaker)
//!
//! 锥形拉回敌人到脚下 + 击飞 + 物理伤害 + 0.5s 眩晕。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::knockback::{CommandKnockback, DisplaceDirection};
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_value};
use lol_core::team::Team;

use crate::sett::Sett;
use crate::sett::buffs::SETT_E_TAG;

/// E 范围半径
pub const SETT_E_RANGE: f32 = 490.0;
/// E 锥形角度
pub const SETT_E_CONE_ANGLE: f32 = 90.0;
/// E 击飞时长
pub const SETT_E_KNOCKUP_DURATION: f32 = 0.5;
/// E 拉回速度
pub const SETT_E_PULL_SPEED: f32 = 1200.0;

pub fn on_sett_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_skill: Query<&Skill>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    let Ok(transform) = q_transform.get(entity) else {
        return;
    };
    let Ok(team) = q_team.get(entity) else {
        return;
    };
    let pos = transform.translation.xz();
    let forward = {
        let f = (trigger.point - pos).normalize_or_zero();
        if f == Vec2::ZERO {
            transform.forward().xz()
        } else {
            f
        }
    };
    let half_angle = SETT_E_CONE_ANGLE.to_radians() / 2.0;

    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let damage = get_skill_value(spell_obj, "damage_calc", skill.level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);

    let mut hit = 0u32;
    for (enemy, enemy_transform) in q_enemies.iter() {
        let Ok(enemy_team) = q_team.get(enemy) else {
            continue;
        };
        if enemy_team == team {
            continue;
        }
        let diff = enemy_transform.translation.xz() - pos;
        let distance = diff.length();
        if distance > SETT_E_RANGE || distance == 0.0 {
            continue;
        }
        let dir = diff.normalize();
        let angle = forward.dot(dir).clamp(-1.0, 1.0).acos();
        if angle > half_angle {
            continue;
        }

        commands.entity(enemy).trigger(|e| CommandKnockback {
            entity: e,
            source: entity,
            distance: SETT_E_RANGE,
            speed: SETT_E_PULL_SPEED,
            duration: Some(SETT_E_KNOCKUP_DURATION),
            direction: DisplaceDirection::Toward,
        });
        if damage > 0.0 {
            commands.entity(enemy).trigger(|e| CommandDamageCreate {
                entity: e,
                source: entity,
                damage_type: DamageType::Physical,
                amount: damage,
                tag: Some(SETT_E_TAG),
            });
        }
        hit += 1;
    }

    debug!("Sett E: 迎面痛击，锥形拉回 {} 个敌人", hit);
}