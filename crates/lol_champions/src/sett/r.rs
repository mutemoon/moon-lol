//! Sett R - 消防官 (The Show Stopper)
//!
//! 抓取最近敌方英雄（475 范围），抱人突进，落地后投掷目标 + 圆形 AoE 砸地 + 物理伤害 + 减速。
//!
//! 流程：
//! 1. Nearest{475} 选取抓取目标 → 挂 GrabbedBy + DebuffKnockup
//! 2. ActionDash 向指针方向突进
//! 3. update_grabbed_entities 每帧同步抓取目标位置到 Sett
//! 4. 落地后：移除 GrabbedBy → PushAway 投掷目标 → Circle AoE 伤害 + 减速

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL4;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::action::displace::{
    ActionDisplace, DisplaceCenter, DisplaceEffect, DisplaceMotion, DisplaceTargetSelection,
    GrabbedBy,
};
use lol_core::buffs::cc_debuffs::DebuffKnockup;
use lol_core::base::buff::BuffOf;
use lol_core::damage::{Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::movement::{EventMovementEnd, MovementSource};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_value};
use lol_core::team::Team;

use crate::sett::Sett;
use crate::sett::buffs::{SettRLandingPending, SETT_R_TAG, SETT_R_SLOW_PERCENT, SETT_R_SLOW_DURATION};

/// R AoE 半径
pub const SETT_R_RADIUS: f32 = 200.0;
/// R 突进最大范围
pub const SETT_R_DASH_MAX_RANGE: f32 = 475.0;
/// R 突进速度
pub const SETT_R_DASH_SPEED: f32 = 1500.0;
/// R 抓取范围
const SETT_R_GRAB_RANGE: f32 = 475.0;
/// R 投掷距离
const SETT_R_THROW_DISTANCE: f32 = 300.0;
/// R 投掷速度
const SETT_R_THROW_SPEED: f32 = 1200.0;
/// R 击飞时长
const SETT_R_KNOCKUP_DURATION: f32 = 0.75;

pub fn on_sett_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    // 1. 选取最近敌方英雄
    let Ok(sett_tf) = q_transform.get(entity) else {
        return;
    };
    let Ok(sett_team) = q_team.get(entity) else {
        return;
    };
    let sett_pos = sett_tf.translation;
    let mut best_dist = SETT_R_GRAB_RANGE;
    let mut grabbed = None;
    for (enemy, enemy_tf, enemy_team) in q_enemies.iter() {
        if enemy_team == sett_team {
            continue;
        }
        let d = enemy_tf.translation.distance(sett_pos);
        if d < best_dist {
            best_dist = d;
            grabbed = Some(enemy);
        }
    }

    // 2. 抓取目标：挂 GrabbedBy + 击飞视觉效果
    if let Some(target) = grabbed {
        commands.entity(target).insert(GrabbedBy { grabber: entity });
        commands
            .entity(target)
            .with_related::<BuffOf>(DebuffKnockup::new(SETT_R_KNOCKUP_DURATION));
    }

    // 3. 自身突进
    commands.trigger(ActionDash {
        entity,
        point: trigger.point,
        move_type: DashMoveType::Pointer {
            max: SETT_R_DASH_MAX_RANGE,
        },
        speed: SETT_R_DASH_SPEED,
    });

    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let damage = get_skill_value(spell_obj, "damage_calc", skill.level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);

    commands.entity(entity).insert(SettRLandingPending {
        damage,
        slow_percent: SETT_R_SLOW_PERCENT,
        slow_duration: SETT_R_SLOW_DURATION,
        grabbed_target: grabbed,
    });
}

/// R 突进落地：投掷被抓目标 + 圆形 AoE 砸地 + 减速。
pub fn on_sett_r_dash_end(
    trigger: On<EventMovementEnd>,
    mut commands: Commands,
    q_pending: Query<&SettRLandingPending>,
) {
    if trigger.event().source != MovementSource::Dash {
        return;
    }

    let entity = trigger.event_target();
    let Ok(pending) = q_pending.get(entity) else {
        return;
    };

    // 投掷被抓目标（PushAway 从落地点向外）
    if let Some(target) = pending.grabbed_target {
        commands.entity(target).remove::<GrabbedBy>();

        // 用 ActionDisplace 投掷目标
        commands.trigger(ActionDisplace {
            entity,
            targets: DisplaceTargetSelection::Explicit(vec![target]),
            motion: DisplaceMotion::PushAway {
                distance: SETT_R_THROW_DISTANCE,
                speed: SETT_R_THROW_SPEED,
            },
            effects: vec![],
            cone_hit_policy: None,
        });
    }

    // 落地 AoE 伤害 + 减速
    let effects = if pending.damage > 0.0 {
        vec![
            DisplaceEffect::Damage {
                amount: pending.damage,
                damage_type: DamageType::Physical,
                tag: Some(SETT_R_TAG),
            },
            DisplaceEffect::Slow {
                percent: pending.slow_percent,
                duration: pending.slow_duration,
            },
        ]
    } else {
        vec![DisplaceEffect::Slow {
            percent: pending.slow_percent,
            duration: pending.slow_duration,
        }]
    };

    commands.trigger(ActionDisplace {
        entity,
        targets: DisplaceTargetSelection::Circle {
            radius: SETT_R_RADIUS,
            center: DisplaceCenter::Caster,
        },
        motion: DisplaceMotion::None,
        effects,
        cone_hit_policy: None,
    });

    commands.entity(entity).remove::<SettRLandingPending>();
}