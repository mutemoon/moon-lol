//! Darius E - 无情立场 (Apprehend)
//!
//! 主动：朝施法方向锥形拉回范围内的敌人到 Darius 脚边，击飞 0.75 秒，
//! 并施加 40% 减速 1 秒。Darius 自身不位移。
//!
//! 组合表达（位移框架组合化）：
//! - 锥形查询复用 `DamageShape::Sector` 的几何（朝向、半径、半角），
//!   但 E 不造成伤害，故不走 `ActionDamage`，而是直接空间查询敌人。
//! - 拉回 = `CommandKnockback { direction: Toward }`（Phase 1.1 原语），
//!   `Toward` 自动钳制不越过 source，故 distance 传范围上限即可拉到脚下；
//!   击飞（`DebuffKnockup`）由 `on_command_knockback` 自动施加。
//! - 减速 = `DebuffSlow`，作为观察者式副作用挂在每个被拉敌人上。
//!
//! 被动护甲穿透待护甲穿透系统支持后实现。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::knockback::{CommandKnockback, DisplaceDirection};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::entities::champion::Champion;
use lol_core::team::Team;

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

/// 施放 Darius E：朝 [point] 方向锥形拉回敌人。
///
/// - 锥形朝向取施法点方向（施法点与自身重合时退回 Transform 面向方向）。
/// - 范围内且在半角内的敌人被 `CommandKnockback{Toward}` 拉到脚下并击飞，
///   额外挂 40% 减速 1 秒。
/// - Darius 自身不位移。
pub fn cast_darius_e(
    commands: &mut Commands,
    entity: Entity,
    _skill_spell: Handle<Spell>,
    point: Vec2,
    q_transform: &Query<&Transform>,
    q_team: &Query<&Team>,
    q_enemies: &Query<(Entity, &Transform), With<Champion>>,
) {
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
    // 锥形朝向：施法点方向；施法点与自身重合时退回面向方向
    let forward = (point - pos).normalize_or_zero();
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
        // 钳制点积避免浮点误差导致 acos NaN
        let angle = forward.dot(dir).clamp(-1.0, 1.0).acos();
        if angle > half_angle {
            continue;
        }

        // 拉回到 Darius 脚边：Toward 钳制不越过 source，distance 传范围上限即可
        commands.entity(enemy).trigger(|e| CommandKnockback {
            entity: e,
            source: entity,
            distance: DARIUS_E_RANGE,
            speed: DARIUS_E_PULL_SPEED,
            duration: Some(DARIUS_E_KNOCKUP_DURATION),
            direction: DisplaceDirection::Toward,
        });
        // 40% 减速 1 秒（击飞由 CommandKnockback 自动施加）
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
