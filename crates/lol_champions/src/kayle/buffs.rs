use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 凯尔被动 - 神圣崛起（攻速加成）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KaylePassive" })]
pub struct BuffKaylePassive {
    pub attackspeed_bonus: f32,
    pub stacks: u8,
    pub timer: Timer,
}

impl BuffKaylePassive {
    pub fn new(attackspeed_bonus: f32, duration: f32) -> Self {
        Self {
            attackspeed_bonus,
            stacks: 1,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn add_stack(&mut self) {
        if self.stacks < 5 {
            self.stacks += 1;
        }
    }
}

/// 凯尔W - 天赐祝福（治疗和移速）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KayleW" })]
pub struct BuffKayleW {
    pub heal_amount: f32,
    pub movespeed_bonus: f32,
    pub timer: Timer,
}

impl BuffKayleW {
    pub fn new(heal_amount: f32, movespeed_bonus: f32, duration: f32) -> Self {
        Self {
            heal_amount,
            movespeed_bonus,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 凯尔E - 星火之刃（强化攻击）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KayleE" })]
pub struct BuffKayleE {
    pub bonus_damage: f32,
    pub timer: Timer,
}

impl BuffKayleE {
    pub fn new(bonus_damage: f32, duration: f32) -> Self {
        Self {
            bonus_damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 凯尔R - 神圣审判（无敌）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "KayleR" })]
pub struct BuffKayleR {
    pub invulnerable: bool,
    pub duration: f32,
    pub timer: Timer,
}

impl BuffKayleR {
    pub fn new(duration: f32) -> Self {
        Self {
            invulnerable: true,
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
