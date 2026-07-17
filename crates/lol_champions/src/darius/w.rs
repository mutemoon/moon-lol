//! Darius W - 致残打击 (Crippling Strike)
//!
//! 攻击重置 + 强化普攻（额外伤害 + 减速）
//!
//! 若击杀目标：返蓝 40 + 减 CD 50%

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::attack::{CommandAttackReset, EventAttackEnd};
use lol_core::base::ability_resource::AbilityResource;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::on_hit::{BuffOnHitBonusDamage, BuffOnHitCounter, BuffOnHitSlow};
use lol_core::life::Health;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot, get_skill_value};

use crate::darius::buffs::{DariusWKillPending, DariusWRefundPending};
use crate::darius::Darius;

pub fn on_darius_w(
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
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandAttackReset { entity });

    // 从 RON 读取总伤害倍率（empowered_attack_damage = total_AD * total_multiplier），
    // 减去 1.0（基础普攻）得到额外伤害比例
    let total_mult = get_skill_value(spell_obj, "empowered_attack_damage", skill.level, |stat| {
        if stat == 2 { 1.0 } else { 0.0 }
    })
    .unwrap_or(1.5);
    let ratio = total_mult - 1.0;

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOnHitCounter::new(1, 1.0))
        .with_related::<BuffOf>(BuffOnHitBonusDamage { flat: 0.0, ratio })
        .with_related::<BuffOf>(BuffOnHitSlow {
            percent: 0.5,
            duration: 1.0,
        });

    // W 击杀标记：命中后若击杀目标则返蓝减 CD
    commands.entity(entity).insert(DariusWRefundPending {
        skill_entity: trigger.skill_entity,
    });
}

/// W 攻击命中后标记延迟检测。
///
/// 因 W 额外伤害经 `commands.trigger(CommandDamageCreate)` 延迟执行，
/// 此时 `health.value` 尚未更新，故将击杀检测推迟到 `FixedUpdate` 中。
pub fn on_darius_w_attack_end(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_pending: Query<&DariusWRefundPending>,
) {
    let attacker = trigger.event_target();
    if q_darius.get(attacker).is_err() {
        return;
    }
    let Ok(pending) = q_pending.get(attacker) else {
        return;
    };

    commands.entity(attacker).insert(DariusWKillPending {
        target: trigger.target,
        skill_entity: pending.skill_entity,
    });
    commands.entity(attacker).remove::<DariusWRefundPending>();
}

/// FixedUpdate 中延迟检测 W 击杀：此时延迟伤害已应用，health.value 已更新。
pub fn check_darius_w_kill(
    mut commands: Commands,
    q_pending: Query<(Entity, &DariusWKillPending)>,
    q_health: Query<&Health>,
    mut q_ability_resource: Query<&mut AbilityResource>,
    mut q_cooldown: Query<&mut CoolDown>,
) {
    for (attacker, pending) in q_pending.iter() {
        if let Ok(health) = q_health.get(pending.target) {
            if health.value <= 0.0 {
                // 返蓝 40
                if let Ok(mut ar) = q_ability_resource.get_mut(attacker) {
                    ar.value = (ar.value + 40.0).min(ar.max);
                }
                // 减 CD 50%
                if let Ok(mut cooldown) = q_cooldown.get_mut(pending.skill_entity) {
                    if let Some(timer) = cooldown.timer.as_mut() {
                        let remaining = timer.remaining_secs() * 0.5;
                        *timer = Timer::from_seconds(remaining.max(0.0), TimerMode::Once);
                    }
                }
            }
        }
        commands.entity(attacker).remove::<DariusWKillPending>();
    }
}