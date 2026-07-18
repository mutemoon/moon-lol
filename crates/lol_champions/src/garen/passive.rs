//! Garen 被动 - 坚韧 (Perseverance)
//!
//! Q 强化普攻命中对目标施加沉默。

use bevy::prelude::*;
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::{BuffOf, Buffs};
use lol_core::buffs::cc_debuffs::DebuffSilence;

use crate::garen::Garen;
use crate::garen::q::BuffGarenQAttack;

/// Garen Q 强化普攻命中：对目标施加沉默（DebuffSilence），并消费 BuffGarenQAttack。
pub fn on_garen_attack_end_silence(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_garen: Query<(), With<Garen>>,
    q_buffs: Query<&Buffs>,
    q_qattack: Query<&BuffGarenQAttack>,
) {
    let attacker = trigger.event_target();
    if q_garen.get(attacker).is_err() {
        return;
    }
    let Ok(buffs) = q_buffs.get(attacker) else {
        return;
    };
    let Some(qattack_entity) = buffs.iter().find(|b| q_qattack.get(*b).is_ok()) else {
        return;
    };
    let Ok(qattack) = q_qattack.get(qattack_entity) else {
        return;
    };
    commands
        .entity(trigger.target)
        .with_related::<BuffOf>(DebuffSilence::new(qattack.silence_duration));
    commands.entity(qattack_entity).despawn();
    debug!("Garen Q 强化普攻命中 {:?}，施加沉默", trigger.target);
}
