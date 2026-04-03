use bevy::prelude::*;
use crate::Buff;

///  Aphelios Q - 狙击枪标记
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "ApheliosCalibrum" })]
pub struct BuffApheliosCalibrum {
    pub damage: f32,
    pub timer: Timer,
}

impl BuffApheliosCalibrum {
    pub fn new(damage: f32, duration: f32) -> Self {
        Self {
            damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

///  Aphelios Q - 重力炮减速
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "ApheliosGravitum" })]
pub struct BuffApheliosGravitum {
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffApheliosGravitum {
    pub fn new(slow_percent: f32, duration: f32) -> Self {
        Self {
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
