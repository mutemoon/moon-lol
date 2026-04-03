use bevy::prelude::*;
use crate::Buff;

/// 卡莉丝塔被动 - 武术姿态（位移）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Kalista MartialPoise" })]
pub struct BuffKalistaMartialPoise {
    pub duration: f32,
    pub timer: Timer,
}

impl BuffKalistaMartialPoise {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 卡莉丝塔E - 撕裂减速
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KalistaE" })]
pub struct BuffKalistaE {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffKalistaE {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 卡莉丝塔R - 命运之召（保护）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KalistaR" })]
pub struct BuffKalistaR {
    pub invulnerable: bool,
    pub duration: f32,
    pub timer: Timer,
}

impl BuffKalistaR {
    pub fn new(duration: f32) -> Self {
        Self {
            invulnerable: true,
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
