//! Fiora Q（前刺 / Lunge）。
//!
//! 语义：向指针方向位移，位移停止后戳刺最近的单位；有敌方英雄时优先戳英雄。
//! 区别于 Riven Q：不对位移路径上的敌人造成碰撞伤害，伤害只在位移结束后
//! 以位移终点为圆心戳刺一次。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL1;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::bounding::Bounding;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::life::Death;
use lol_core::movement::{EventMovementEnd, MovementSource};
use lol_core::skill::get_skill_value;
use lol_core::team::Team;

use crate::fiora::Fiora;

/// Q 位移最大距离（向指针方向突刺）。
const FIORA_Q_DASH_MAX: f32 = 300.0;
const FIORA_Q_DASH_SPEED: f32 = 1000.0;
/// 位移停止后，以位移终点为圆心的戳刺索敌半径。
const FIORA_Q_STRIKE_RADIUS: f32 = 200.0;
/// 伤害公式键名（与 `FioraQ.ron` 中 `calculations` 的键一致）。
const FIORA_Q_DAMAGE_KEY: &str = "total_damage";

/// Q 施法后挂上的临时标记：位移结束时尚未戳刺。
///
/// 携带技能法术句柄与等级，供位移结束 observer 计算伤害数值。
/// 位移结束 observer 触发后立即移除，保证一次 Q 只戳刺一次。
#[derive(Component)]
pub struct FioraQPending {
    skill: Handle<Spell>,
    level: usize,
}

/// Q 施法：先纯位移（不对路径上的敌人造成碰撞伤害），位移结束后再戳刺最近单位。
pub fn cast_fiora_q(
    commands: &mut Commands,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
    level: usize,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });

    // 纯位移：damage 为 None，不产生 DashDamageComponent，
    // 因此不会像 Riven Q 那样对路径上的敌人造成碰撞伤害。
    commands.trigger(ActionDash {
        entity,
        point,
        skill: Handle::default(),
        move_type: DashMoveType::Pointer {
            max: FIORA_Q_DASH_MAX,
        },
        damage: None,
        speed: FIORA_Q_DASH_SPEED,
    });

    // 标记：等位移结束 observer 再戳刺。
    commands.entity(entity).insert(FioraQPending {
        skill: skill_spell,
        level,
    });
}

/// 位移结束后戳刺最近单位：有敌方英雄时优先戳英雄，否则戳最近的任意敌方单位。
pub fn on_fiora_q_dash_end(
    trigger: On<EventMovementEnd>,
    mut commands: Commands,
    res_assets_spell_object: Res<Assets<Spell>>,
    q_fiora: Query<(&Transform, &Team, &Damage, &FioraQPending), With<Fiora>>,
    q_target: Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&Champion>,
            Option<&Bounding>,
        ),
        Without<Death>,
    >,
) {
    // 只处理位移（Dash）结束，其它移动结束直接忽略，避免走位结束误触发戳刺。
    if trigger.source != MovementSource::Dash {
        return;
    }

    let entity = trigger.event_target();
    let Ok((transform, team, damage, pending)) = q_fiora.get(entity) else {
        // 不是处于 Q 待戳刺状态的 Fiora，忽略。
        return;
    };

    let fiora_pos = transform.translation;

    // 在戳刺半径内寻找：最近的敌方英雄 / 最近的任意敌方单位。
    // 命中判定以敌人边缘为准：有效范围 = 戳刺半径 + 目标碰撞半径，
    // 即 `dist - target_radius <= STRIKE_RADIUS`，而非仅看中心点。
    let mut nearest_champion: Option<(Entity, f32)> = None;
    let mut nearest_any: Option<(Entity, f32)> = None;
    for (target, target_transform, target_team, champion, bounding) in q_target.iter() {
        if target_team == team {
            continue;
        }
        let dist = target_transform.translation.distance(fiora_pos);
        let target_radius = bounding.map_or(0.0, |b| b.radius);
        let gap = dist - target_radius; // 距敌人边缘的距离
        if gap > FIORA_Q_STRIKE_RADIUS {
            continue;
        }
        if nearest_any.map_or(true, |(_, g)| gap < g) {
            nearest_any = Some((target, gap));
        }
        if champion.is_some() && nearest_champion.map_or(true, |(_, g)| gap < g) {
            nearest_champion = Some((target, gap));
        }
    }

    // 有英雄优先戳英雄，否则戳最近单位。
    if let Some((target, _)) = nearest_champion.or(nearest_any) {
        if let Some(spell_object) = res_assets_spell_object.get(&pending.skill) {
            let amount = get_skill_value(spell_object, FIORA_Q_DAMAGE_KEY, pending.level, |stat| {
                if stat == 2 { damage.0 } else { 0.0 }
            })
            .unwrap_or(0.0);

            commands.entity(target).trigger(|e| CommandDamageCreate {
                entity: e,
                source: entity,
                damage_type: DamageType::Physical,
                amount,
                tag: None,
            });
        }
    }

    commands.entity(entity).remove::<FioraQPending>();
}
