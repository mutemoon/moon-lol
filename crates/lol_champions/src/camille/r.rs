//! Camille R（海克斯最后通牒 / Hextech Ultimatum）。
//!
//! 跃向目标，跃起期间不可选中，落地后标记目标，普攻命中被标记目标时造成
//! 额外魔法伤害（`RPercentCurrentHPDamage`% 当前生命值），持续 `RDuration`。
//! 落地时击退附近其他敌人。

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_base::animation_names::ANIM_SPELL4;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::action::displace::{ActionDisplace, DisplaceMotion, DisplaceTargetSelection};
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::{Buff, BuffOf, Buffs};
use lol_core::buffs::cc_debuffs::ImmuneToCC;
use lol_core::buffs::damage_reduction::BuffDamageReduction;
use lol_core::damage::{CommandDamageCreate, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::movement::{EventMovementEnd, MovementSource};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_data_value};
use lol_core::team::Team;

use crate::camille::Camille;
use crate::camille::buffs::CamilleRLeapPending;

/// R 冲刺速度。
const CAMILLE_R_DASH_SPEED: f32 = 1200.0;
/// R 停止半径。
const CAMILLE_R_STOP_RADIUS: f32 = 130.0;
/// R 击退其他敌人的距离。
const CAMILLE_R_KNOCKBACK_DISTANCE: f32 = 400.0;
/// R 击退速度。
const CAMILLE_R_KNOCKBACK_SPEED: f32 = 800.0;
/// R 搜索附近敌人的半径（用于击退）。
const CAMILLE_R_KNOCKBACK_RADIUS: f32 = 450.0;

/// R 标记：记录额外伤害百分比与持续时间。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "CamilleRMark" })]
pub struct BuffCamilleRMark {
    pub percent: f32,
    pub timer: Timer,
}

impl BuffCamilleRMark {
    pub fn new(percent: f32, duration: f32) -> Self {
        Self {
            percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 标记目标（R 施法时调用）。
pub fn apply_camille_r_mark(commands: &mut Commands, target: Entity, percent: f32, duration: f32) {
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffCamilleRMark::new(percent, duration));
}

/// R 施放：跃向最近敌方英雄，跃起期间不可选中。
pub fn on_camille_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<&Team, With<Camille>>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
    q_transform: Query<&Transform, With<Camille>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    let Ok(camille_team) = q_camille.get(entity) else {
        return;
    };
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    let level = skill.level;
    let percent = get_skill_data_value(spell_obj, "RPercentCurrentHPDamage", level).unwrap_or(2.0);
    let duration = get_skill_data_value(spell_obj, "RDuration", level).unwrap_or(1.75);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    // 找最近敌方英雄
    let origin = q_transform
        .get(entity)
        .map(|t| t.translation.xz())
        .unwrap_or_default();
    let nearest = q_enemies
        .iter()
        .filter(|(_, _, team)| **team != *camille_team)
        .min_by(|a, b| {
            let da = (a.1.translation.xz() - origin).length_squared();
            let db = (b.1.translation.xz() - origin).length_squared();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(e, _, _)| e);

    let Some(target) = nearest else {
        return;
    };

    // 标记目标（立即生效，不等达阵）
    apply_camille_r_mark(&mut commands, target, percent, duration);

    // 跃起不可选中
    commands.entity(entity).insert(ImmuneToCC);
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffDamageReduction::new(1.0, None));

    // 追踪标记
    commands.entity(entity).insert(CamilleRLeapPending {
        target,
        percent,
        duration,
    });

    // 跃向目标
    commands.trigger(ActionDash {
        entity,
        point: trigger.point,
        move_type: DashMoveType::Entity {
            target,
            stop_radius: CAMILLE_R_STOP_RADIUS,
        },
        speed: CAMILLE_R_DASH_SPEED,
    });
}

/// R 抵达目标：清除不可选中，击退附近其他敌人。
pub fn on_camille_r_arrival(
    trigger: On<EventMovementEnd>,
    mut commands: Commands,
    q_camille: Query<&Team, With<Camille>>,
    q_pending: Query<&CamilleRLeapPending>,
    q_transform: Query<&Transform, With<Camille>>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
    q_buffs: Query<&Buffs>,
    q_damage_reduction: Query<&BuffDamageReduction>,
) {
    if trigger.event().source != MovementSource::Dash {
        return;
    }
    let entity = trigger.event_target();
    let Ok(camille_team) = q_camille.get(entity) else {
        return;
    };
    let Ok(pending) = q_pending.get(entity) else {
        return;
    };

    // 清除不可选中
    commands.entity(entity).remove::<ImmuneToCC>();
    if let Ok(buffs) = q_buffs.get(entity) {
        for b in buffs.iter() {
            if q_damage_reduction.get(b).is_ok() {
                commands.entity(b).despawn();
            }
        }
    }

    // 击退附近其他敌人（除 R 目标外）
    let landing = q_transform
        .get(entity)
        .map(|t| t.translation.xz())
        .unwrap_or_default();
    let others: Vec<Entity> = q_enemies
        .iter()
        .filter(|(e, tf, team)| {
            **team != *camille_team
                && *e != pending.target
                && tf.translation.xz().distance(landing) < CAMILLE_R_KNOCKBACK_RADIUS
        })
        .map(|(e, _, _)| e)
        .collect();

    if !others.is_empty() {
        commands.trigger(ActionDisplace {
            entity,
            targets: DisplaceTargetSelection::Explicit(others),
            motion: DisplaceMotion::PushAway {
                distance: CAMILLE_R_KNOCKBACK_DISTANCE,
                speed: CAMILLE_R_KNOCKBACK_SPEED,
            },
            effects: vec![],
            cone_hit_policy: None,
        });
    }

    commands.entity(entity).remove::<CamilleRLeapPending>();
}

/// 普攻命中被标记目标：造成额外魔法伤害（% 当前生命值）。
pub fn on_camille_r_attack_end(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_buffs: Query<&Buffs>,
    q_mark: Query<&BuffCamilleRMark>,
    q_health: Query<&Health>,
) {
    let attacker = trigger.event_target();
    if q_camille.get(attacker).is_err() {
        return;
    }
    let target = trigger.target;
    let Ok(buffs) = q_buffs.get(target) else {
        return;
    };
    let Some(percent) = buffs
        .iter()
        .find_map(|b| q_mark.get(b).ok().map(|m| m.percent))
    else {
        return;
    };
    let Ok(hp) = q_health.get(target) else {
        return;
    };
    let bonus = hp.value * percent / 100.0;
    if bonus <= 0.0 {
        return;
    }
    commands.entity(target).trigger(|e| CommandDamageCreate {
        entity: e,
        source: attacker,
        damage_type: DamageType::Magic,
        amount: bonus,
        tag: None,
    });
}

/// R 标记计时：到期回收标记。
pub fn update_camille_r_mark(
    mut commands: Commands,
    mut q: Query<(Entity, &mut BuffCamilleRMark)>,
    time: Res<Time<Fixed>>,
) {
    for (e, mut mark) in q.iter_mut() {
        mark.timer.tick(time.delta());
        if mark.timer.is_finished() {
            commands.entity(e).despawn();
        }
    }
}
