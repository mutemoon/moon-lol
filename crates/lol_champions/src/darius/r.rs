//! Darius R - 诺克萨斯断头台 (Noxian Guillotine)
//!
//! 跃向目标造成真实伤害，伤害随目标出血层数提升。
//! 若击杀目标：R1/R2 可在 6 秒内再次施放，R3 完全刷新冷却。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL4;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::{BuffOf, Buffs};
use lol_core::buffs::cc_debuffs::ImmuneToCC;
use lol_core::buffs::damage_reduction::BuffDamageReduction;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::movement::{EventMovementEnd, MovementSource};
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot, SkillRecastWindow, get_skill_data_value, get_skill_value};
use lol_core::team::Team;

use crate::darius::buffs::{DariusRLeapPending, DariusRKillPending, BuffDariusBleed};
use crate::darius::Darius;

/// R 冲刺速度。
const DARIUS_R_DASH_SPEED: f32 = 1800.0;
/// R 冲刺停止半径（近战攻击距离）。
const DARIUS_R_STOP_RADIUS: f32 = 130.0;
/// R 搜索范围。
const DARIUS_R_RANGE: f32 = 475.0;

/// R 施放：跃向最近敌方英雄，跃起期间不可选中。
pub fn on_darius_r(
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
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    // 范围内最近敌方英雄
    let origin = q_transform
        .get(entity)
        .map(|t| t.translation.xz())
        .unwrap_or_default();
    let own_team = q_team.get(entity).ok();
    let mut nearest: Option<(Entity, f32)> = None;
    for (enemy, enemy_tf) in q_enemies.iter() {
        if enemy == entity || q_team.get(enemy).ok() == own_team {
            continue;
        }
        let d = enemy_tf.translation.xz().distance(origin);
        if d > DARIUS_R_RANGE {
            continue;
        }
        if nearest.map(|(_, nd)| d < nd).unwrap_or(true) {
            nearest = Some((enemy, d));
        }
    }
    let Some(target) = nearest.map(|(e, _)| e) else {
        return;
    };

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    // 跃起不可选中
    commands.entity(entity).insert(ImmuneToCC);
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffDamageReduction::new(1.0, None));

    // 冲刺到目标
    commands.trigger(ActionDash {
        entity,
        point: trigger.point,
        move_type: DashMoveType::Entity {
            target,
            stop_radius: DARIUS_R_STOP_RADIUS,
        },
        speed: DARIUS_R_DASH_SPEED,
    });

    // 追踪标记
    commands.entity(entity).insert(DariusRLeapPending {
        target,
        skill_entity: trigger.skill_entity,
        skill_level: skill.level as u8,
    });
}

/// R 抵达目标：结算伤害。
pub fn on_darius_r_arrival(
    trigger: On<EventMovementEnd>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_pending: Query<&DariusRLeapPending>,
    q_damage: Query<&Damage>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
    q_buffs: Query<&Buffs>,
    q_bleed: Query<&BuffDariusBleed>,
    q_damage_reduction: Query<&BuffDamageReduction>,
) {
    if trigger.event().source != MovementSource::Dash {
        return;
    }
    let entity = trigger.event_target();
    if q_darius.get(entity).is_err() {
        return;
    }
    let Ok(pending) = q_pending.get(entity) else {
        return;
    };
    let target = pending.target;

    // 清除不可选中
    commands.entity(entity).remove::<ImmuneToCC>();
    if let Ok(buffs) = q_buffs.get(entity) {
        for b in buffs.iter() {
            if q_damage_reduction.get(b).is_ok() {
                commands.entity(b).despawn();
            }
        }
    }

    // 读 spell 计算伤害
    let Ok(skill) = q_skill.get(pending.skill_entity) else {
        commands.entity(entity).remove::<DariusRLeapPending>();
        return;
    };
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        commands.entity(entity).remove::<DariusRLeapPending>();
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let base = get_skill_value(spell_obj, "damage", skill.level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);

    // 目标出血层数 → 每层 RDamagePercentPerHemoStack
    let stacks = q_buffs
        .get(target)
        .ok()
        .map(|buffs| {
            buffs
                .iter()
                .filter_map(|b| q_bleed.get(b).ok())
                .next()
                .map(|bl| bl.stacks)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    let per_stack = get_skill_data_value(spell_obj, "RDamagePercentPerHemoStack", skill.level)
        .unwrap_or(0.2);
    let amount = base * (1.0 + per_stack * stacks as f32);

    // 应用伤害
    commands.entity(target).trigger(|e| CommandDamageCreate {
        entity: e,
        source: entity,
        damage_type: DamageType::True,
        amount,
        tag: None,
    });

    // 延迟击杀检测
    commands.entity(entity).insert(DariusRKillPending {
        skill_entity: pending.skill_entity,
        target,
        skill_level: pending.skill_level,
    });
    commands.entity(entity).remove::<DariusRLeapPending>();
}

/// FixedUpdate 中延迟检测 R 击杀：此时延迟伤害已应用。
pub fn check_darius_r_kill(
    mut commands: Commands,
    q_pending: Query<(Entity, &DariusRKillPending)>,
    q_health: Query<&Health>,
    mut q_cooldown: Query<&mut CoolDown>,
) {
    for (attacker, pending) in q_pending.iter() {
        let killed = q_health
            .get(pending.target)
            .is_ok_and(|h| h.value <= 0.0);

        if killed {
            // 清除冷却
            if let Ok(mut cooldown) = q_cooldown.get_mut(pending.skill_entity) {
                cooldown.timer = None;
            }
            // R1/R2 添加重施窗口（6 秒），R3 已完全刷新
            if pending.skill_level < 3 {
                commands
                    .entity(pending.skill_entity)
                    .insert(SkillRecastWindow::new(1, 1, 6.0));
            }
        }

        commands.entity(attacker).remove::<DariusRKillPending>();
    }
}