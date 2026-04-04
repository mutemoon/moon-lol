use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 千珏被动 - 印记
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KindredMark" })]
pub struct BuffKindredMark {
    pub stacks: u8,
    pub timer: Timer,
}

impl BuffKindredMark {
    pub fn new(stacks: u8, duration: f32) -> Self {
        Self {
            stacks,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 千珏W - Wolf的狂乱（区域攻击）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KindredW" })]
pub struct BuffKindredW {
    pub damage: f32,
    pub duration: f32,
    pub timer: Timer,
}

impl BuffKindredW {
    pub fn new(damage: f32, duration: f32) -> Self {
        Self {
            damage,
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 千珏E - 骑乘恐惧减速
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KindredE" })]
pub struct BuffKindredE {
    pub stacks: u8,
    pub slow_percent: f32,
    pub timer: Timer,
}

impl BuffKindredE {
    pub fn new(stacks: u8, slow_percent: f32, duration: f32) -> Self {
        Self {
            stacks,
            slow_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn add_stack(&mut self) {
        if self.stacks < 3 {
            self.stacks += 1;
        }
    }
}

/// 千珏R - Lamb的庇护（保护）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KindredR" })]
pub struct BuffKindredR {
    pub min_health_percent: f32,
    pub heal_amount: f32,
    pub duration: f32,
    pub timer: Timer,
}

impl BuffKindredR {
    pub fn new(min_health_percent: f32, heal_amount: f32, duration: f32) -> Self {
        Self {
            min_health_percent,
            heal_amount,
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
