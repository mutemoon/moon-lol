//! Darius 被动 - 出血 (Hemorrhage) + 诺克萨斯之力 (Noxian Might)

use bevy::prelude::*;
use lol_base::render_cmd::{CommandSkinParticleDespawn, CommandSkinParticleSpawn};
use lol_core::base::buff::{BuffOf, Buffs};
use lol_core::damage::{CommandDamageCreate, Damage, DamageType, EventDamageCreate};

use crate::darius::buffs::{
    BuffDariusBleed, BuffDariusMight, DARIUS_BLEED_AD_RATIO, DARIUS_BLEED_MAX_STACKS,
    DARIUS_NOXIAN_MIGHT_AD_RATIO,
};

/// 标记出血 DoT 伤害，避免再次叠层。
pub const DARIUS_BLEED_DOT_TAG: u32 = 1;

/// 诺克萨斯之力自身光效键。
const DARIUS_MIGHT_PARTICLE: &str = "Darius_P_enraged";

/// 出血层数计数器粒子键（1~5 层）。
fn hemo_counter_key(stacks: u8) -> &'static str {
    match stacks {
        1 => "Darius_hemo_counter_01",
        2 => "Darius_hemo_counter_02",
        3 => "Darius_hemo_counter_03",
        4 => "Darius_hemo_counter_04",
        _ => "Darius_hemo_counter_05",
    }
}

/// 监听 Darius 造成的伤害，给目标叠加出血；叠满 5 层触发诺克萨斯之力。
pub fn on_darius_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_darius: Query<(), With<super::Darius>>,
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
    // Q 内圈也不叠层（wiki 约定）
    if trigger.event().tag == Some(DARIUS_BLEED_DOT_TAG)
        || trigger.event().tag == Some(super::buffs::DARIUS_Q_INNER_TAG)
    {
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
                let old_stacks = bleed.stacks;
                let was_below_max = bleed.stacks < DARIUS_BLEED_MAX_STACKS;
                bleed.add_stack();
                if was_below_max && bleed.stacks >= DARIUS_BLEED_MAX_STACKS {
                    reached_five = true;
                }
                // 层数变化：换计数器粒子（先撤旧层再挂新层）
                commands.trigger(CommandSkinParticleDespawn {
                    entity: target,
                    hash: hemo_counter_key(old_stacks).to_string(),
                    resolver_entity: Some(source),
                });
                commands.trigger(CommandSkinParticleSpawn {
                    entity: target,
                    hash: hemo_counter_key(bleed.stacks).to_string(),
                    rotation: None,
                    resolver_entity: Some(source),
                });
                found_existing = true;
                break;
            }
        }
    }
    if !found_existing {
        commands
            .entity(target)
            .with_related::<BuffOf>(BuffDariusBleed::new(source));
        commands.trigger(CommandSkinParticleSpawn {
            entity: target,
            hash: hemo_counter_key(1).to_string(),
            rotation: None,
            resolver_entity: Some(source),
        });
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
            // 诺克萨斯之力光效（到期由 update_darius_might 撤除）
            commands.trigger(CommandSkinParticleSpawn {
                entity: source,
                hash: DARIUS_MIGHT_PARTICLE.to_string(),
                rotation: None,
                resolver_entity: None,
            });
        }
    }
}

/// 出血 DoT：每周期造成 0.3*AD*层数 物理伤害，持续 5 秒后清除。
pub fn update_darius_bleed(
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
            // 出血到期：撤除当前层数计数器粒子
            commands.trigger(CommandSkinParticleDespawn {
                entity: bo.0,
                hash: hemo_counter_key(bleed.stacks).to_string(),
                resolver_entity: Some(bleed.source),
            });
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
pub fn update_darius_might(
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
        // 到期：撤除诺克萨斯之力光效
        commands.trigger(CommandSkinParticleDespawn {
            entity: darius,
            hash: DARIUS_MIGHT_PARTICLE.to_string(),
            resolver_entity: None,
        });
        commands.entity(entity).despawn();
    }
}
