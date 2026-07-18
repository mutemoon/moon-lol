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
use lol_core::attack::CommandAttackReset;
use lol_core::base::bounding::Bounding;
use lol_core::base::direction::is_in_direction;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::life::Death;
use lol_core::movement::{EventMovementEnd, MovementSource};
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillSlot, get_skill_data_value, get_skill_value,
};
use lol_core::team::Team;

use crate::fiora::Fiora;
use crate::fiora::passive::Vital;

/// Q 位移最大距离（向指针方向突刺）。
const FIORA_Q_DASH_MAX: f32 = 300.0;
const FIORA_Q_DASH_SPEED: f32 = 1000.0;
/// 位移停止后，以位移终点为圆心的戳刺索敌半径。
const FIORA_Q_STRIKE_RADIUS: f32 = 200.0;
/// 伤害公式键名（与 `FioraQ.ron` 中 `calculations` 的键一致）。
const FIORA_Q_DAMAGE_KEY: &str = "total_damage";
/// 命中要害时伤害翻倍倍率（wiki：击中要害伤害翻倍）。
const FIORA_Q_VITAL_MULT: f32 = 2.0;
/// 冷却退还没收键名（与 `FioraQ.ron` 中 `dataValues` 的键一致）。
const FIORA_Q_CD_REFUND_KEY: &str = "CDRefundPercent";

/// Q 施法后挂上的临时标记：位移结束时尚未戳刺。
#[derive(Component)]
pub struct FioraQPending {
    skill: Handle<Spell>,
    level: usize,
    skill_entity: Entity,
}

pub fn on_fiora_q(
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

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });

    commands.trigger(ActionDash {
        entity,
        point: trigger.point,
        move_type: DashMoveType::Pointer {
            max: FIORA_Q_DASH_MAX,
        },
        speed: FIORA_Q_DASH_SPEED,
    });

    commands.trigger(CommandAttackReset { entity });

    commands.entity(entity).insert(FioraQPending {
        skill: skill.spell.clone(),
        level: skill.level,
        skill_entity: trigger.skill_entity,
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
            Option<&Vital>,
        ),
        Without<Death>,
    >,
    mut q_cooldown: Query<&mut CoolDown>,
) {
    if trigger.source != MovementSource::Dash {
        return;
    }

    let entity = trigger.event_target();
    let Ok((transform, team, damage, pending)) = q_fiora.get(entity) else {
        return;
    };

    let fiora_pos = transform.translation;
    let fiora_xz = fiora_pos.xz();

    let mut nearest_champion: Option<(Entity, f32)> = None;
    let mut nearest_any: Option<(Entity, f32)> = None;
    for (target, target_transform, target_team, champion, bounding, _) in q_target.iter() {
        if target_team == team {
            continue;
        }
        let dist = target_transform.translation.distance(fiora_pos);
        let target_radius = bounding.map_or(0.0, |b| b.radius);
        let gap = dist - target_radius;
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

    if let Some((target, _)) = nearest_champion.or(nearest_any) {
        let vital_hit = q_target
            .get(target)
            .ok()
            .and_then(|(_, t_transform, _, _, _, vital)| {
                let vital = vital?;
                Some(
                    vital.is_active()
                        && is_in_direction(
                            fiora_xz,
                            t_transform.translation.xz(),
                            &vital.direction,
                        ),
                )
            })
            .unwrap_or(false);
        let multiplier = if vital_hit { FIORA_Q_VITAL_MULT } else { 1.0 };

        if let Some(spell_object) = res_assets_spell_object.get(&pending.skill) {
            let amount = get_skill_value(spell_object, FIORA_Q_DAMAGE_KEY, pending.level, |stat| {
                if stat == 2 { damage.0 } else { 0.0 }
            })
            .unwrap_or(0.0)
                * multiplier;

            commands.entity(target).trigger(|e| CommandDamageCreate {
                entity: e,
                source: entity,
                damage_type: DamageType::Physical,
                amount,
                tag: None,
            });

            let refund = get_skill_data_value(spell_object, FIORA_Q_CD_REFUND_KEY, pending.level)
                .unwrap_or(0.0);
            if refund > 0.0 {
                if let Ok(mut cooldown) = q_cooldown.get_mut(pending.skill_entity) {
                    if let Some(timer) = cooldown.timer.as_mut() {
                        let remaining = timer.remaining_secs() * (1.0 - refund);
                        *timer = Timer::from_seconds(remaining.max(0.0), TimerMode::Once);
                    }
                }
            }
        }
    }

    commands.entity(entity).remove::<FioraQPending>();
}
