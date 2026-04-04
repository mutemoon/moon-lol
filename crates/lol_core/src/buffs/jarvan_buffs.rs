use bevy::prelude::*;

use crate::base::buff::Buff;

/// 嘉文四世被动 - 战争律动
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "JarvanPassive" })]
pub struct BuffJarvanPassive {
    pub timer: Timer,
}

impl BuffJarvanPassive {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        }
    }
}
