//! Mordekaiser E - 断魂一拽 (Death's Grasp)
//!
//! 朝施法方向释放死亡之爪，将前方锥形（半径 550）内的敌人向莫德凯撒拽回 250，
//! 并造成魔法伤害（基础 + 40% AP）。命中叠 1 层被动 Darkness（由被动观察者统一处理）。
//!
//! 组合表达（位移框架组合化，参照 Darius E）：
//! - 锥形查询复用空间几何，E 本身直接对命中敌人触发伤害与位移。
//! - 拽回 = `CommandKnockback { direction: Toward, distance: 250 }`，
//!   `Toward` 自动钳制不越过 source，故 250 即"向自身拉近 250"。
//!
//! 被动法术穿透（ron `MagicPen` 0.025-0.175）待法穿系统支持后实现。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::knockback::{CommandKnockback, DisplaceDirection};
use lol_core::damage::{AbilityPower, CommandDamageCreate, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::skill::{get_skill_data_value, get_skill_value};
use lol_core::team::Team;

/// E 锥形半径（拽回范围，ron `MaxDistance` = 550）
pub const MORDE_E_RANGE: f32 = 550.0;
/// E 锥形半角（度）
pub const MORDE_E_CONE_HALF_ANGLE: f32 = 50.0;
/// E 拽回距离（ron `KnockTowardsDistance` = 250）
pub const MORDE_E_PULL_DISTANCE: f32 = 250.0;
/// E 拽回速度
pub const MORDE_E_PULL_SPEED: f32 = 1500.0;
/// E 拽回持续时间（秒）
pub const MORDE_E_PULL_DURATION: f32 = 0.4;

/// 施放 Mordekaiser E：朝 [point] 方向锥形拽回敌人并造成魔法伤害。
pub fn cast_mordekaiser_e(
    commands: &mut Commands,
    entity: Entity,
    _skill_spell: Handle<Spell>,
    point: Vec2,
    skill_level: usize,
    spell_obj: &Spell,
    q_transform: &Query<&Transform>,
    q_team: &Query<&Team>,
    q_enemies: &Query<(Entity, &Transform), With<Champion>>,
    q_ap: &Query<&AbilityPower>,
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
    let half_angle = MORDE_E_CONE_HALF_ANGLE.to_radians();

    let max_distance =
        get_skill_data_value(spell_obj, "MaxDistance", skill_level).unwrap_or(MORDE_E_RANGE);
    let pull_distance = get_skill_data_value(spell_obj, "KnockTowardsDistance", skill_level)
        .unwrap_or(MORDE_E_PULL_DISTANCE);
    let ap = q_ap.get(entity).map(|a| a.0).unwrap_or(0.0);
    let damage = get_skill_value(spell_obj, "total_damage", skill_level, |stat| {
        if stat == 0 { ap } else { 0.0 }
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
        if distance > max_distance || distance == 0.0 {
            continue;
        }
        let dir = diff.normalize();
        let angle = forward.dot(dir).clamp(-1.0, 1.0).acos();
        if angle > half_angle {
            continue;
        }

        // 拽回：Toward 钳制不越过 source，distance 即拉近量
        commands.entity(enemy).trigger(|e| CommandKnockback {
            entity: e,
            source: entity,
            distance: pull_distance,
            speed: MORDE_E_PULL_SPEED,
            duration: Some(MORDE_E_PULL_DURATION),
            direction: DisplaceDirection::Toward,
        });
        // 魔法伤害（命中叠被动由 on_mordekaiser_damage_hit 统一处理）
        commands.entity(enemy).trigger(|e| CommandDamageCreate {
            entity: e,
            source: entity,
            damage_type: DamageType::Magic,
            amount: damage,
            tag: None,
        });
        hit += 1;
    }

    debug!("莫德凯撒 E 断魂一拽：锥形拽回 {} 个敌人", hit);
}
