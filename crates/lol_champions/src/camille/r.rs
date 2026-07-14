//! Camille R（海克斯最后通牒 / Hextech Ultimatum）。
//!
//! 简化实现：R 标记目标，普攻命中被标记目标时造成额外魔法伤害
//! （`RPercentCurrentHPDamage`% 当前生命值），持续 `RDuration`。
//! 区域禁锢 / 击退等机制按位移框架 Phase 4.2 暂缓。

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::{Buff, BuffOf, Buffs};
use lol_core::damage::{CommandDamageCreate, DamageType};
use lol_core::life::Health;

use crate::camille::Camille;

/// R 标记：记录额外伤害百分比与持续时间。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "CamilleRMark" })]
pub struct BuffCamilleRMark {
    /// 额外伤害占当前生命值百分比（ron RPercentCurrentHPDamage，1 级 = 2.0）。
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
