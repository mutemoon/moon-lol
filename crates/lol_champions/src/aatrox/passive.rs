//! Aatrox 被动 - 死亡镰刀 (Deathbringer Stance)
//!
//! 就绪时下次普攻附带目标最大生命值 15% 额外魔法伤害 + 治疗；冷却 22s。

use bevy::prelude::*;
use lol_core::attack::EventAttackEnd;
use lol_core::damage::{CommandDamageCreate, DamageType};
use lol_core::life::Health;
use lol_core::team::Team;

use crate::aatrox::Aatrox;
use crate::aatrox::buffs::AatroxPassiveState;

/// 被动伤害标签
pub const AATROX_P_TAG: u32 = 10;

/// 普攻命中时：若被动就绪，对目标造成其最大生命值 15% 额外魔法伤害并治疗自身，随后进入冷却。
pub fn on_aatrox_attack_end(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    mut q_aatrox: Query<(&mut AatroxPassiveState, &Team), With<Aatrox>>,
    q_team: Query<&Team>,
    q_health: Query<&Health>,
) {
    let attacker = trigger.event_target();
    let Ok((mut passive, caster_team)) = q_aatrox.get_mut(attacker) else {
        return;
    };
    if !passive.ready {
        return;
    }
    let target = trigger.event().target;
    let Ok(target_team) = q_team.get(target) else {
        return;
    };
    if target_team == caster_team {
        return;
    }
    let Ok(target_hp) = q_health.get(target) else {
        return;
    };

    let bonus = target_hp.max * AatroxPassiveState::DAMAGE_RATIO;
    if bonus > 0.0 {
        commands.entity(target).trigger(|e| CommandDamageCreate {
            entity: e,
            source: attacker,
            damage_type: DamageType::Magic,
            amount: bonus,
            tag: Some(AATROX_P_TAG),
        });
    }
    let heal = bonus * AatroxPassiveState::HEAL_RATIO;
    if heal > 0.0 {
        commands
            .entity(attacker)
            .queue(move |mut e: EntityWorldMut| {
                if let Some(mut health) = e.get_mut::<Health>() {
                    health.value = (health.value + heal).min(health.max);
                }
            });
    }

    passive.ready = false;
    passive.timer.reset();
}

/// 被动冷却倒计时：到时再次就绪。
pub fn update_aatrox_passive(
    time: Res<Time>,
    mut q_aatrox: Query<&mut AatroxPassiveState, With<Aatrox>>,
) {
    for mut passive in q_aatrox.iter_mut() {
        if passive.ready {
            continue;
        }
        passive.timer.tick(time.delta());
        if passive.timer.just_finished() {
            passive.ready = true;
        }
    }
}