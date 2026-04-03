use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 卢锡安被动 - 圣光枪弹
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LucianPassive" })]
pub struct BuffLucianPassive {
    pub bonus_damage: f32,
    pub timer: Timer,
}

impl BuffLucianPassive {
    pub fn new(bonus_damage: f32, duration: f32) -> Self {
        Self {
            bonus_damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 卢锡安W - 炽热魔弹标记
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LucianW" })]
pub struct BuffLucianW {
    pub movespeed_bonus: f32,
    pub timer: Timer,
}

impl BuffLucianW {
    pub fn new(movespeed_bonus: f32, duration: f32) -> Self {
        Self {
            movespeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 卢锡安E - 无情追击（位移）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LucianE" })]
pub struct BuffLucianE {
    pub cooldown_reduction: f32,
    pub timer: Timer,
}

impl BuffLucianE {
    pub fn new(cooldown_reduction: f32, duration: f32) -> Self {
        Self {
            cooldown_reduction,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 卢锡安R - 圣枪洗礼
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LucianR" })]
pub struct BuffLucianR {
    pub damage_per_shot: f32,
    pub timer: Timer,
}

impl BuffLucianR {
    pub fn new(damage_per_shot: f32, duration: f32) -> Self {
        Self {
            damage_per_shot,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
