//! Camille 被动（自适应防御 / Adaptive Defenses）。
//!
//! 对敌方英雄的普攻命中后，获得一个基于最大生命值的护盾（持续 2s）。
//! 护盾用通用 `BuffShieldWhite`（抵挡全类型伤害）承载，2s 到期由
//! `BuffCamillePassiveTimer` 追踪并统一回收。

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::{Buff, BuffOf};
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::EventDamageCreate;
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::team::Team;

use crate::camille::Camille;
use crate::camille::w::CAMILLE_W_OUTER_TAG;

/// 被动护盾占最大生命值比例（wiki：6%）。
pub const CAMILLE_PASSIVE_SHIELD_RATIO: f32 = 0.06;
/// 被动护盾持续时间（wiki：2s）。
pub const CAMILLE_PASSIVE_DURATION: f32 = 2.0;

/// 被动护盾计时器：到期回收护盾。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "CamillePassive" })]
pub struct BuffCamillePassiveTimer {
    pub timer: Timer,
}

impl BuffCamillePassiveTimer {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(CAMILLE_PASSIVE_DURATION, TimerMode::Once),
        }
    }
}

/// 普攻命中敌方英雄时获得护盾。
///
/// 刷新语义：命中前先清除该角色上既存的被动护盾与计时器，再生成新的一份，
/// 保证至多存在一对护盾+计时器（旧计时器过期时不会误杀刷新后的护盾）。
pub fn on_camille_attack_end(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_camille: Query<(&Team, &Health), With<Camille>>,
    q_target: Query<&Team, With<Champion>>,
    q_buffof: Query<(Entity, &BuffOf)>,
    q_shield: Query<&BuffShieldWhite>,
    q_timer: Query<&BuffCamillePassiveTimer>,
) {
    let attacker = trigger.event_target();
    let target = trigger.target;
    let Ok((camille_team, hp)) = q_camille.get(attacker) else {
        return;
    };
    let Ok(target_team) = q_target.get(target) else {
        return;
    };
    if target_team == camille_team {
        return;
    }

    // 清除既存的被动护盾与计时器（刷新）
    for (e, bo) in q_buffof.iter() {
        if bo.0 != attacker {
            continue;
        }
        if q_shield.get(e).is_ok() || q_timer.get(e).is_ok() {
            commands.entity(e).despawn();
        }
    }

    let shield_amount = hp.max * CAMILLE_PASSIVE_SHIELD_RATIO;
    commands
        .entity(attacker)
        .with_related::<BuffOf>(BuffShieldWhite::new(shield_amount))
        .with_related::<BuffOf>(BuffCamillePassiveTimer::new());
}

/// 被动护盾计时：到期回收护盾与计时器。
pub fn update_camille_passive(
    mut commands: Commands,
    mut q_timer: Query<(Entity, &BuffOf, &mut BuffCamillePassiveTimer)>,
    q_buffof: Query<(Entity, &BuffOf)>,
    q_shield: Query<&BuffShieldWhite>,
    time: Res<Time<Fixed>>,
) {
    for (timer_entity, buffof, mut timer) in q_timer.iter_mut() {
        timer.timer.tick(time.delta());
        if !timer.timer.is_finished() {
            continue;
        }
        let camille = buffof.0;
        for (e, bo) in q_buffof.iter() {
            if bo.0 == camille && q_shield.get(e).is_ok() {
                commands.entity(e).despawn();
            }
        }
        commands.entity(timer_entity).despawn();
    }
}

/// 监听 Camille 造成的伤害，给目标施加减速。
pub fn on_camille_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
) {
    // 仅 W 外圈命中施加减速
    if trigger.tag != Some(CAMILLE_W_OUTER_TAG) {
        return;
    }
    let source = trigger.source;
    if q_camille.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.8, 2.0));
}
