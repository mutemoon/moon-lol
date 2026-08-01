use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_base::render_cmd::CommandSkinParticleDespawn;
use lol_core::base::buff::{Buff, BuffOf};

/// 不稳标记 -- 挂在被标记的敌方实体上。
///
/// 由 E2 / R 命中施加（持续 `MarkDuration` 秒）。
/// 处于不稳状态的敌人被 Q 命中时，Q 冷却刷新（核心追击机制）。
///
/// `caster` 用于到期时撤除目标身上的标记粒子（fixed_update 里拿不到施法者）。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "IreliaUnsteady" })]
pub struct DebuffIreliaUnsteady {
    pub caster: Entity,
    pub timer: Timer,
}

impl DebuffIreliaUnsteady {
    pub fn new(caster: Entity, duration: f32) -> Self {
        Self {
            caster,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// FixedUpdate：tick 不稳标记计时器，到期销毁并撤除标记粒子。
pub fn update_irelia_unsteady(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut q_unsteady: Query<(Entity, &BuffOf, &mut DebuffIreliaUnsteady)>,
) {
    let mut expired = Vec::new();
    for (entity, buffof, mut unsteady) in q_unsteady.iter_mut() {
        unsteady.timer.tick(time.delta());
        if unsteady.timer.is_finished() {
            expired.push((entity, buffof.0, unsteady.caster));
        }
    }
    for (entity, target, caster) in expired {
        commands.trigger(CommandSkinParticleDespawn {
            entity: target,
            hash: "Irelia_Q_Mark".to_string(),
            resolver_entity: Some(caster),
        });
        commands.entity(entity).despawn();
    }
}
