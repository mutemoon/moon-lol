use bevy::prelude::*;
use lol_core::base::buff::Buff;
use lol_core::buffs::damage_reduction::BuffDamageReduction;

/// 不稳标记 —— 挂在被标记的敌方实体上
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

/// Irelia W 蓄力减伤
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "IreliaW" })]
pub struct BuffIreliaW {
    pub damage_reduction: f32,
    pub timer: Timer,
}

impl BuffIreliaW {
    pub fn new(damage_reduction: f32, duration: f32) -> Self {
        Self {
            damage_reduction,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// Irelia E2/R 释放后标记状态 —— 挂在 Irelia 自身，表示接下来的伤害会标记目标
#[derive(Component, Debug, Clone)]
pub struct IreliaMarkActive {
    pub timer: Timer,
}

impl IreliaMarkActive {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

pub trait IreliaBuff {
    fn irelia_w(damage_reduction: f32, duration: f32) -> (BuffIreliaW, BuffDamageReduction);
}

impl IreliaBuff for BuffDamageReduction {
    fn irelia_w(damage_reduction: f32, duration: f32) -> (BuffIreliaW, BuffDamageReduction) {
        (
            BuffIreliaW::new(damage_reduction, duration),
            BuffDamageReduction::new(damage_reduction, None),
        )
    }
}
