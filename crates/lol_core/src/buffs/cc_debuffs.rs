use bevy::prelude::*;

use crate::base::buff::Buff;
use crate::movement::MovementBlock;

/// 眩晕
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Stun" })]
pub struct DebuffStun {
    pub timer: Timer,
}

impl DebuffStun {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 更新眩晕计时，结束后移除眩晕和移动阻塞
pub fn update_debuff_stun(
    mut commands: Commands,
    mut q_stun: Query<(Entity, &mut DebuffStun)>,
    time: Res<Time<Fixed>>,
) {
    for (entity, mut stun) in q_stun.iter_mut() {
        stun.timer.tick(time.delta());
        if stun.timer.is_finished() {
            commands.entity(entity).remove::<DebuffStun>();
            commands.entity(entity).remove::<MovementBlock>();
        }
    }
}

/// 减速
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Slow" })]
pub struct DebuffSlow {
    pub percent: f32, // 0.0-1.0
    pub timer: Timer,
}

impl DebuffSlow {
    pub fn new(percent: f32, duration: f32) -> Self {
        Self {
            percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 沉默
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Silence" })]
pub struct DebuffSilence {
    pub timer: Timer,
}

impl DebuffSilence {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 恐惧
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Fear" })]
pub struct DebuffFear {
    pub timer: Timer,
}

impl DebuffFear {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
