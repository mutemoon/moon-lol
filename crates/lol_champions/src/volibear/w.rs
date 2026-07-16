//! Volibear W - 疯狂撕咬 (Frenzied Maul)
//!
//! W1 咬最近敌人 + 标记 8s；W2 重施对已标记目标 1.5x 伤害 + 治疗。

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::base::buff::{BuffOf, Buffs};
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, get_skill_cast_radius,
    get_skill_data_value, get_skill_value,
};
use lol_core::team::Team;

use crate::volibear::Volibear;
use crate::volibear::buffs::DebuffVolibearWMark;

/// W 伤害标签
pub const VOLIBEAR_W_TAG: u32 = 1;
/// W 重施窗口（秒）
pub const VOLIBEAR_W_RECAST_WINDOW: f32 = 2.0;
/// W 标记持续时长（秒）
pub const VOLIBEAR_W_MARK_DURATION: f32 = 8.0;

/// 在 caster 周围 range 内寻找最近的敌方英雄。
fn nearest_enemy(
    caster_pos: Vec2,
    range: f32,
    caster_team: &Team,
    q_enemies: &Query<(Entity, &Transform), With<Champion>>,
    q_team: &Query<&Team>,
) -> Option<Entity> {
    let mut best: Option<(Entity, f32)> = None;
    for (enemy, transform) in q_enemies.iter() {
        let Ok(enemy_team) = q_team.get(enemy) else {
            continue;
        };
        if enemy_team == caster_team {
            continue;
        }
        let dist = transform.translation.xz().distance(caster_pos);
        if dist <= range && best.map(|(_, d)| dist < d).unwrap_or(true) {
            best = Some((enemy, dist));
        }
    }
    best.map(|(e, _)| e)
}

/// enemy 是否挂有来自 source 的 W 标记。
fn is_marked_by(
    enemy: Entity,
    source: Entity,
    q_buffs: &Query<&Buffs>,
    q_mark: &Query<&DebuffVolibearWMark>,
) -> bool {
    let Ok(buffs) = q_buffs.get(enemy) else {
        return false;
    };
    buffs
        .iter()
        .find_map(|b| q_mark.get(b).ok())
        .map(|mark| mark.source == source)
        .unwrap_or(false)
}

pub fn on_volibear_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
    q_buffs: Query<&Buffs>,
    q_mark: Query<&DebuffVolibearWMark>,
    q_health: Query<&Health>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_volibear.get(entity).is_err() {
        return;
    }
    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    let caster_pos = q_transform
        .get(entity)
        .map(|t| t.translation.xz())
        .unwrap_or(Vec2::ZERO);
    let Ok(caster_team) = q_team.get(entity) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let max_hp = q_health.get(entity).map(|h| h.max).unwrap_or(0.0);
    let current_hp = q_health.get(entity).map(|h| h.value).unwrap_or(0.0);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    let stage = recast.map(|w| w.stage).unwrap_or(1);
    let total_damage = get_skill_value(spell_obj, "total_damage", skill.level, |stat| {
        if stat == 2 {
            ad
        } else if stat == 12 {
            max_hp
        } else {
            0.0
        }
    })
    .unwrap_or(0.0);
    let cast_range = get_skill_cast_radius(spell_obj, skill.level).unwrap_or(325.0);

    if stage == 1 {
        // W1：开启重施窗口 + 咬最近敌人 + 标记
        commands.entity(trigger.skill_entity).insert(SkillRecastWindow::new(
            2,
            2,
            VOLIBEAR_W_RECAST_WINDOW,
        ));
        if let Some(enemy) = nearest_enemy(caster_pos, cast_range, caster_team, &q_enemies, &q_team) {
            if total_damage > 0.0 {
                commands.entity(enemy).trigger(|e| CommandDamageCreate {
                    entity: e,
                    source: entity,
                    damage_type: DamageType::Physical,
                    amount: total_damage,
                    tag: Some(VOLIBEAR_W_TAG),
                });
            }
            commands
                .entity(enemy)
                .with_related::<BuffOf>(DebuffVolibearWMark::new(entity, VOLIBEAR_W_MARK_DURATION));
        }
    } else {
        // W2：移除重施 + 重置冷却 + 咬最近敌人（已标记则 1.5x + 治疗）
        commands.entity(trigger.skill_entity).remove::<SkillRecastWindow>();
        commands.entity(trigger.skill_entity).insert(CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        });

        let w2_mult =
            get_skill_data_value(spell_obj, "W2DamageMultiplier", skill.level).unwrap_or(1.5);
        let base_heal = get_skill_data_value(spell_obj, "BaseHeal", skill.level).unwrap_or(5.0);
        let heal_percent =
            get_skill_data_value(spell_obj, "HealPercent", skill.level).unwrap_or(0.05);

        if let Some(enemy) = nearest_enemy(caster_pos, cast_range, caster_team, &q_enemies, &q_team) {
            let marked = is_marked_by(enemy, entity, &q_buffs, &q_mark);
            let dmg = total_damage * if marked { w2_mult } else { 1.0 };
            if dmg > 0.0 {
                commands.entity(enemy).trigger(|e| CommandDamageCreate {
                    entity: e,
                    source: entity,
                    damage_type: DamageType::Physical,
                    amount: dmg,
                    tag: Some(VOLIBEAR_W_TAG),
                });
            }
            if marked {
                let missing = (max_hp - current_hp).max(0.0);
                let heal = base_heal + heal_percent * missing;
                if heal > 0.0 {
                    commands.entity(entity).queue(move |mut e: EntityWorldMut| {
                        if let Some(mut health) = e.get_mut::<Health>() {
                            health.value = (health.value + heal).min(health.max);
                        }
                    });
                }
            }
        }
    }
}