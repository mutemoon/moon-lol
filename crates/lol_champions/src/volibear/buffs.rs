use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 沃利贝尔W标记 —— 挂在被W1命中的目标上
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "VolibearWMark" })]
pub struct DebuffVolibearWMark {
    pub source: Entity,
    pub timer: Timer,
}

impl DebuffVolibearWMark {
    pub fn new(source: Entity, duration: f32) -> Self {
        Self {
            source,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
