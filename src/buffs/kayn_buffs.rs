use bevy::prelude::*;

use crate::Buff;

/// Kayn R 寄生 —— 挂在被标记的目标身上
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KaynR" })]
pub struct DebuffKaynR {
    pub source: Entity, // Kayn's entity
    pub timer: Timer,
}

impl DebuffKaynR {
    pub fn new(source: Entity, duration: f32) -> Self {
        Self {
            source,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// Kayn R 激活状态 —— 挂在 Kayn 自身，表示正在寄生中（不可选中）
#[derive(Component, Debug, Clone)]
pub struct BuffKaynRActive {
    pub target: Entity, // 被寄生的目标
    pub timer: Timer,
}

impl BuffKaynRActive {
    pub fn new(target: Entity, duration: f32) -> Self {
        Self {
            target,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
