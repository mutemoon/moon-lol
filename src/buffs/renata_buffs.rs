use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 瑞纳斯Q - 铁绑鞭（减速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RenataQ" })]
pub struct BuffRenataQ {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffRenataQ {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 瑞纳斯W - 广域忠护（攻速加成）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RenataW" })]
pub struct BuffRenataW {
    pub attackspeed_bonus: f32,
    pub timer: Timer,
}

impl BuffRenataW {
    pub fn new(attackspeed_bonus: f32, duration: f32) -> Self {
        Self {
            attackspeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 瑞纳斯R - 终极毒梦（眩晕）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RenataR" })]
pub struct BuffRenataR {
    pub stun_duration: f32,
    pub timer: Timer,
}

impl BuffRenataR {
    pub fn new(stun_duration: f32, duration: f32) -> Self {
        Self {
            stun_duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
