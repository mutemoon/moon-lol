//! Sett 被动 - 沙场战心 (Grit & Punch State)
//!
//! - 左右拳交替，右拳（偶数次）附带 0.55×AD 额外物理伤害。
//! - 灰心：受到伤害时累积（上限 50% 最大生命），脱战 4 秒后衰减。

use bevy::prelude::*;
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::{DebuffSlow, DebuffStun};
use lol_core::damage::{CommandDamageCreate, Damage, DamageType, EventDamageCreate};
use lol_core::life::Health;

use crate::sett::Sett;
use crate::sett::buffs::{
    SettGrit, SettPunchState, SETT_E_STUN_DURATION, SETT_E_TAG, SETT_GRIT_CAP_RATIO,
    SETT_R_SLOW_DURATION, SETT_R_SLOW_PERCENT, SETT_R_TAG, SETT_PASSIVE_RIGHT_PUNCH_RATIO,
};

/// 监听 Sett 造成的伤害：仅 E 标签眩晕、仅 R 标签减速（Q/被动/W 不附带 CC）。
pub fn on_sett_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
) {
    let source = trigger.source;
    if q_sett.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    match trigger.event().tag {
        Some(SETT_E_TAG) => {
            commands
                .entity(target)
                .with_related::<BuffOf>(DebuffStun::new(SETT_E_STUN_DURATION));
        }
        Some(SETT_R_TAG) => {
            commands
                .entity(target)
                .with_related::<BuffOf>(DebuffSlow::new(SETT_R_SLOW_PERCENT, SETT_R_SLOW_DURATION));
        }
        _ => {}
    }
}

/// 被动（沙场战心）：每次普攻命中左右拳交替，右拳（偶数次）附带 0.55×AD 额外物理伤害。
pub fn on_sett_attack_end(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_punch: Query<&SettPunchState>,
    q_damage: Query<&Damage>,
) {
    let attacker = trigger.event_target();
    if q_sett.get(attacker).is_err() {
        return;
    }
    let target = trigger.target;

    let count = q_punch.get(attacker).map(|p| p.count).unwrap_or(0);
    let new_count = count + 1;

    commands
        .entity(attacker)
        .queue(move |mut e: EntityWorldMut| {
            if let Some(mut punch) = e.get_mut::<SettPunchState>() {
                punch.count = new_count;
            } else {
                e.insert(SettPunchState { count: new_count });
            }
        });

    // 右拳（偶数次）附带 0.55×AD 物理伤害
    if new_count % 2 == 0 {
        let ad = q_damage.get(attacker).map(|d| d.0).unwrap_or(0.0);
        let bonus = ad * SETT_PASSIVE_RIGHT_PUNCH_RATIO;
        if bonus > 0.0 {
            commands.entity(target).trigger(|e| CommandDamageCreate {
                entity: e,
                source: attacker,
                damage_type: DamageType::Physical,
                amount: bonus,
                tag: None,
            });
        }
    }
}

/// 监听 Sett 受到的伤害：累积"灰心"（= final_damage，上限 50% 最大生命），重置脱战计时。
pub fn on_sett_damage_taken(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_health: Query<&Health>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }
    let final_dmg = trigger.event().damage_result.final_damage;
    if final_dmg <= 0.0 {
        return;
    }
    let max_hp = q_health.get(entity).map(|h| h.max).unwrap_or(0.0);
    let cap = SETT_GRIT_CAP_RATIO * max_hp;

    commands.entity(entity).queue(move |mut e: EntityWorldMut| {
        if let Some(mut grit) = e.get_mut::<SettGrit>() {
            grit.stored = (grit.stored + final_dmg).min(cap);
            grit.combat_timer.reset();
        } else {
            let mut grit = SettGrit::new();
            grit.stored = final_dmg.min(cap);
            e.insert(grit);
        }
    });
}

/// 灰心脱战衰减：脱战计时器结束后清零灰心。
pub fn update_sett_grit(time: Res<Time>, mut q_grit: Query<&mut SettGrit, With<Sett>>) {
    for mut grit in q_grit.iter_mut() {
        grit.combat_timer.tick(time.delta());
        if grit.combat_timer.is_finished() {
            grit.stored = 0.0;
        }
    }
}