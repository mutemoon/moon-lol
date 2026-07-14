//! Q - 利刃冲击 (Bladesurge)
//!
//! 冲向敌人造成物理伤害；命中"不稳"状态的敌人时刷新冷却（核心追击机制）。
//! 伤害在施法瞬间直接结算到距施法点最近的敌方英雄，位移由 `ActionDash` 承载。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL1;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::Buffs;
use lol_core::damage::{CommandDamageCreate, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, get_skill_value};
use lol_core::team::Team;

use crate::irelia::buffs::DebuffIreliaUnsteady;
use crate::irelia::{IRELIA_Q_DAMAGE_TAG, IRELIA_Q_RANGE};

/// Q 施法距离（ron `castRange`）
const IRELIA_Q_DASH_MAX: f32 = 250.0;

pub fn cast_irelia_q(
    commands: &mut Commands,
    entity: Entity,
    point: Vec2,
    skill_entity: Entity,
    skill_level: usize,
    spell: &Spell,
    q_enemies: &Query<(Entity, &Transform, &Team), With<Champion>>,
    team: Team,
    q_buffs: &Query<&Buffs>,
    q_unsteady: &Query<&DebuffIreliaUnsteady>,
    cooldown: &CoolDown,
    ad: f32,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q 重置普攻
    commands.trigger(CommandAttackReset { entity });

    // 命中距施法点最近的敌方英雄，直接结算物理伤害
    let nearest = q_enemies
        .iter()
        .filter(|(_, _, t)| **t != team)
        .filter(|(_, tf, _)| tf.translation.xz().distance(point) <= IRELIA_Q_RANGE)
        .min_by(|a, b| {
            let da = a.1.translation.xz().distance(point);
            let db = b.1.translation.xz().distance(point);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });

    if let Some((target, _, _)) = nearest {
        let amount = get_skill_value(spell, "champion_damage", skill_level, |stat| {
            if stat == 2 { ad } else { 0.0 }
        })
        .unwrap_or(0.0);

        commands.entity(target).trigger(|e| CommandDamageCreate {
            entity: e,
            source: entity,
            damage_type: DamageType::Physical,
            amount,
            tag: Some(IRELIA_Q_DAMAGE_TAG),
        });

        // 命中"不稳"目标：刷新 Q 冷却（timer=None 表示立即就绪）。
        // 不稳标记是 target 的子 buff，需经 Buffs 列表查询。
        let is_unsteady = q_buffs
            .get(target)
            .map(|buffs| buffs.iter().any(|b| q_unsteady.get(b).is_ok()))
            .unwrap_or(false);
        if is_unsteady {
            commands.entity(skill_entity).insert(CoolDown {
                duration: cooldown.duration,
                timer: None,
            });
        }
    }

    // 位移（朝施法点冲刺）
    commands.trigger(ActionDash {
        entity,
        point,
        move_type: DashMoveType::Pointer {
            max: IRELIA_Q_DASH_MAX,
        },
        speed: 800.0,
    });
}
