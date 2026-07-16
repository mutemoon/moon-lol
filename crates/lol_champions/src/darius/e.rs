//! Darius E - 无情立场 (Apprehend)
//!
//! 主动：朝施法方向锥形拉回范围内的敌人到 Darius 脚边，击飞 0.75 秒，
//! 并施加 40% 减速 1 秒。Darius 自身不位移。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::knockback::{CommandKnockback, DisplaceDirection};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};
use lol_core::team::Team;

use crate::darius::Darius;

/// E 锥形范围（拉回距离，wiki: 535）
pub const DARIUS_E_RANGE: f32 = 535.0;
/// E 锥形张角（度）
pub const DARIUS_E_CONE_ANGLE: f32 = 90.0;
/// E 击飞持续时间（秒，wiki: 固定 0.75）
pub const DARIUS_E_KNOCKUP_DURATION: f32 = 0.75;
/// E 拉回速度
pub const DARIUS_E_PULL_SPEED: f32 = 1200.0;
/// E 减速强度（40%）
pub const DARIUS_E_SLOW_PERCENT: f32 = 0.4;
/// E 减速持续时间（秒）
pub const DARIUS_E_SLOW_DURATION: f32 = 1.0;

pub fn on_darius_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_skill: Query<&Skill>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
) {
    let entity = trigger.event_target();
    if q_darius.get(entity).is_err() {
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

    let Ok(transform) = q_transform.get(entity) else {
        return;
    };
    let Ok(team) = q_team.get(entity) else {
        return;
    };

    let pos = transform.translation.xz();
    let forward = (trigger.point - pos).normalize_or_zero();
    let forward = if forward == Vec2::ZERO {
        transform.forward().xz()
    } else {
        forward
    };
    let half_angle = DARIUS_E_CONE_ANGLE.to_radians() / 2.0;

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
        if distance > DARIUS_E_RANGE || distance == 0.0 {
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
            distance: DARIUS_E_RANGE,
            speed: DARIUS_E_PULL_SPEED,
            duration: Some(DARIUS_E_KNOCKUP_DURATION),
            direction: DisplaceDirection::Toward,
        });
        commands
            .entity(enemy)
            .with_related::<BuffOf>(DebuffSlow::new(
                DARIUS_E_SLOW_PERCENT,
                DARIUS_E_SLOW_DURATION,
            ));
        hit += 1;
    }

    debug!("Darius E: 无情立场，锥形拉回 {} 个敌人 + 击飞 + 减速", hit);
}