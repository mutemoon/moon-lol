use bevy::prelude::*;
use crate::Buff;

/// 卡西奥佩娅被动 - 蛇眼优雅
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "CassioPassive" })]
pub struct BuffCassioPassive {
    pub timer: Timer,
}

impl BuffCassioPassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        }
    }
}

/// 卡西奥佩娅中毒效果
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "CassioPoison" })]
pub struct BuffCassioPoison {
    pub timer: Timer,
}

impl BuffCassioPoison {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}
