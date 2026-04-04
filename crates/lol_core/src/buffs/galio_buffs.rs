use bevy::prelude::*;

use crate::base::buff::Buff;

/// 加里奥被动 - 巨石碾击
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "GalioPassive" })]
pub struct BuffGalioPassive {
    pub timer: Timer,
}

impl BuffGalioPassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        }
    }
}

/// 加里奥W - 杜朗石像
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "GalioW" })]
pub struct BuffGalioW {
    pub timer: Timer,
}

impl BuffGalioW {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(4.0, TimerMode::Once),
        }
    }
}
