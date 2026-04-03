use bevy::prelude::*;
use crate::Buff;

/// 阿卡丽被动 - 刺客印记
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AkaliPassive" })]
pub struct BuffAkaliPassive {
    pub ring_created: bool,
}

impl BuffAkaliPassive {
    pub fn new() -> Self {
        Self { ring_created: false }
    }
}

/// 阿卡丽W - 暮光之刃（烟雾持续）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AkaliW" })]
pub struct BuffAkaliW {
    pub timer: Timer,
}

impl BuffAkaliW {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        }
    }
}

/// 阿卡丽W - 隐身状态
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AkaliStealth" })]
pub struct BuffAkaliStealth {
    pub timer: Timer,
}

impl BuffAkaliStealth {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Once),
        }
    }
}
