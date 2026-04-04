use bevy::prelude::*;

use crate::base::buff::Buff;

/// 盲僧W2 - 铁意/生命偷取
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "LeeSinIronWill" })]
pub struct BuffLeeSinIronWill {
    pub lifesteal: f32,
    pub spell_vamp: f32,
    pub duration: f32,
    pub timer: Timer,
}

impl BuffLeeSinIronWill {
    pub fn new(lifesteal: f32, spell_vamp: f32, duration: f32) -> Self {
        Self {
            lifesteal,
            spell_vamp,
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        self.timer.is_finished()
    }
}
