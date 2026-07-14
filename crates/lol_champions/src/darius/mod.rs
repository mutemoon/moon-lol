pub mod buffs;
pub mod e;
pub mod q;

#[cfg(test)]
mod e_tests;
#[cfg(test)]
mod passive_tests;
#[cfg(test)]
mod q_tests;
#[cfg(test)]
mod tests;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL2, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::{BuffOf, Buffs};
use lol_core::buffs::on_hit::{BuffOnHitBonusDamage, BuffOnHitCounter, BuffOnHitSlow};
use lol_core::damage::{CommandDamageCreate, Damage, DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_value};
use lol_core::team::Team;

use crate::darius::buffs::{
    BuffDariusBleed, BuffDariusMight, DARIUS_BLEED_AD_RATIO, DARIUS_BLEED_MAX_STACKS,
    DARIUS_NOXIAN_MIGHT_AD_RATIO,
};
use crate::darius::e::cast_darius_e as execute_darius_e;
use crate::darius::q::cast_darius_q as execute_darius_q;

#[derive(Default)]
pub struct PluginDarius;

impl Plugin for PluginDarius {
    fn build(&self, app: &mut App) {
        app.add_observer(on_darius_q);
        app.add_observer(on_darius_w);
        app.add_observer(on_darius_e);
        app.add_observer(on_darius_r);
        app.add_observer(on_darius_damage_hit);
        app.add_systems(FixedUpdate, update_darius_bleed);
        app.add_systems(FixedUpdate, update_darius_might);
    }
}

#[derive(Component, Reflect, Default)]
#[require(Champion, Name = Name::new("Darius"))]
#[reflect(Component)]
pub struct Darius;

/// 标记出血 DoT 伤害，避免 `on_darius_damage_hit` 再次叠层。
pub const DARIUS_BLEED_DOT_TAG: u32 = 1;
/// R 斩杀范围（最近敌方英雄）。
const DARIUS_R_RANGE: f32 = 400.0;
/// 每层出血使 R 伤害提升 20%（法术数据 RDamagePercentPerHemoStack）。
const DARIUS_R_DAMAGE_PER_STACK: f32 = 0.2;

fn on_darius_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_darius.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    execute_darius_q(
        &mut commands,
        entity,
        skill.spell.clone(),
        skill.level,
        spell_obj,
        trigger.point,
    );
}

fn on_darius_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_darius.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    cast_darius_w(&mut commands, entity, skill.level);
}

fn on_darius_e(
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

    execute_darius_e(
        &mut commands,
        entity,
        skill.spell.clone(),
        trigger.point,
        &q_transform,
        &q_team,
        &q_enemies,
    );
}

fn on_darius_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
    q_buffs: Query<&Buffs>,
    q_bleed: Query<&BuffDariusBleed>,
    q_damage: Query<&Damage>,
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

    // 解析 R 基础伤害公式（r_base_damage + 0.75*AD）
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let base = res_spells
        .get(&skill.spell)
        .and_then(|s| {
            get_skill_value(
                s,
                "damage",
                skill.level,
                |stat| if stat == 2 { ad } else { 0.0 },
            )
        })
        .unwrap_or(0.0);

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

    // 目标出血层数 -> 每层 +20% R 伤害
    let stacks = nearest
        .and_then(|(enemy, _)| q_buffs.get(enemy).ok())
        .map(|buffs| {
            buffs
                .iter()
                .filter_map(|b| q_bleed.get(b).ok())
                .next()
                .map(|bl| bl.stacks)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    let amount = base * (1.0 + DARIUS_R_DAMAGE_PER_STACK * stacks as f32);

    cast_darius_r(&mut commands, entity, nearest.map(|(e, _)| e), amount);
}

fn cast_darius_w(commands: &mut Commands, entity: Entity, level: usize) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W 组合：攻击重置 + 强化普攻（额外伤害 + 减速）
    commands.trigger(CommandAttackReset { entity });

    // 伤害比例：40% + 5% per level (40/45/50/55/60%)
    let ratio = 0.35 + (level as f32) * 0.05;

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOnHitCounter::new(1, 1.0))
        .with_related::<BuffOf>(BuffOnHitBonusDamage { flat: 0.0, ratio })
        .with_related::<BuffOf>(BuffOnHitSlow {
            percent: 0.5,
            duration: 1.0,
        });

    debug!(
        "Darius W: 强化普攻（伤害比例 {:.0}% + 减速 50% 1s）",
        ratio * 100.0
    );
}

fn cast_darius_r(commands: &mut Commands, entity: Entity, target: Option<Entity>, amount: f32) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R 斩杀：直接对最近敌方英雄造成物理伤害，伤害随目标出血层数 +20%/层
    if let Some(target) = target {
        commands.entity(target).trigger(|e| CommandDamageCreate {
            entity: e,
            source: entity,
            damage_type: DamageType::Physical,
            amount,
            tag: None,
        });
    }
}

/// 监听 Darius 造成的伤害，给目标叠加出血；叠满 5 层触发诺克萨斯之力。
fn on_darius_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_damage: Query<&Damage>,
    q_buffs: Query<&Buffs>,
    mut q_bleed: Query<&mut BuffDariusBleed>,
    q_might: Query<&BuffDariusMight>,
) {
    let source = trigger.source;
    if q_darius.get(source).is_err() {
        return;
    }
    // DoT 伤害不再叠加出血，避免无限叠层
    if trigger.event().tag == Some(DARIUS_BLEED_DOT_TAG) {
        return;
    }
    let target = trigger.event_target();
    let ad = q_damage.get(source).map(|d| d.0).unwrap_or(0.0);

    // 查找目标已有的出血 buff：有则叠层，无则新建
    let mut reached_five = false;
    let mut found_existing = false;
    if let Ok(buffs) = q_buffs.get(target) {
        for b in buffs.iter() {
            if let Ok(mut bleed) = q_bleed.get_mut(b) {
                let was_below_max = bleed.stacks < DARIUS_BLEED_MAX_STACKS;
                bleed.add_stack();
                if was_below_max && bleed.stacks >= DARIUS_BLEED_MAX_STACKS {
                    reached_five = true;
                }
                found_existing = true;
                break;
            }
        }
    }
    if !found_existing {
        commands
            .entity(target)
            .with_related::<BuffOf>(BuffDariusBleed::new(source));
    }

    // 叠满 5 层 -> 诺克萨斯之力（+50% AD），已存在则不重复施加
    if reached_five {
        let has_might = q_buffs
            .get(source)
            .map(|buffs| buffs.iter().any(|b| q_might.get(b).is_ok()))
            .unwrap_or(false);
        if !has_might {
            let bonus = ad * DARIUS_NOXIAN_MIGHT_AD_RATIO;
            commands
                .entity(source)
                .with_related::<BuffOf>(BuffDariusMight::new(bonus));
            commands.entity(source).insert(Damage(ad + bonus));
        }
    }
}

/// 出血 DoT：每周期造成 0.3*AD*层数 物理伤害，持续 5 秒后清除。
fn update_darius_bleed(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut q_bleed: Query<(Entity, &mut BuffDariusBleed, &BuffOf)>,
    q_damage: Query<&Damage>,
) {
    let dt = time.delta();
    let mut expired = Vec::new();
    for (entity, mut bleed, bo) in q_bleed.iter_mut() {
        bleed.duration_timer.tick(dt);
        if bleed.duration_timer.is_finished() {
            expired.push(entity);
            continue;
        }
        bleed.tick_timer.tick(dt);
        if bleed.tick_timer.just_finished() {
            let ad = q_damage.get(bleed.source).map(|d| d.0).unwrap_or(0.0);
            let amount = DARIUS_BLEED_AD_RATIO * ad * bleed.stacks as f32;
            commands.trigger(CommandDamageCreate {
                entity: bo.0,
                source: bleed.source,
                damage_type: DamageType::Physical,
                amount,
                tag: Some(DARIUS_BLEED_DOT_TAG),
            });
        }
    }
    for entity in expired {
        commands.entity(entity).despawn();
    }
}

/// 诺克萨斯之力到期：移除 buff 并恢复 AD。
fn update_darius_might(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut q_might: Query<(Entity, &mut BuffDariusMight, &BuffOf)>,
    mut q_damage: Query<&mut Damage>,
) {
    let dt = time.delta();
    let mut expired = Vec::new();
    for (entity, mut might, bo) in q_might.iter_mut() {
        might.timer.tick(dt);
        if might.timer.is_finished() {
            expired.push((entity, bo.0, might.ad_bonus));
        }
    }
    for (entity, darius, bonus) in expired {
        if let Ok(mut d) = q_damage.get_mut(darius) {
            d.0 -= bonus;
        }
        commands.entity(entity).despawn();
    }
}
