use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_core::base::buff::Buff;

/// 不稳标记 -- 挂在被标记的敌方实体上。
///
/// 由 E2 / R 命中施加（持续 `MarkDuration` 秒）。
/// 处于不稳状态的敌人被 Q 命中时，Q 冷却刷新（核心追击机制）。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "IreliaUnsteady" })]
pub struct DebuffIreliaUnsteady {
    pub timer: Timer,
}

impl DebuffIreliaUnsteady {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// FixedUpdate：tick 不稳标记计时器，到期销毁。
pub fn update_irelia_unsteady(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut q_unsteady: Query<(Entity, &mut DebuffIreliaUnsteady)>,
) {
    let mut expired = Vec::new();
    for (entity, mut unsteady) in q_unsteady.iter_mut() {
        unsteady.timer.tick(time.delta());
        if unsteady.timer.is_finished() {
            expired.push(entity);
        }
    }
    for entity in expired {
        commands.entity(entity).despawn();
    }
}
