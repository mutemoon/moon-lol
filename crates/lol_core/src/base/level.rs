use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 经验值组件，记录单位死亡时掉落经验
#[derive(Component, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct ExperienceDrop {
    /// 死亡时掉落的经验值
    pub exp_given_on_death: f32,
    /// 经验掉落范围半径
    pub experience_radius: f32,
}

impl Default for ExperienceDrop {
    fn default() -> Self {
        Self {
            exp_given_on_death: 0.0,
            experience_radius: 0.0,
        }
    }
}

#[derive(Component, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Level {
    pub value: u32,
    pub experience: u32,
    pub experience_to_next_level: u32,
}

impl Default for Level {
    fn default() -> Self {
        Self {
            value: 1,
            experience: 0,
            experience_to_next_level: 100,
        }
    }
}

#[derive(EntityEvent)]
pub struct EventLevelUp {
    pub entity: Entity,
    pub level: u32,
    pub delta: u32,
}

impl Level {
    pub fn add_experience(&mut self, experience: u32) -> u32 {
        self.experience += experience;
        let mut levels_gained = 0;
        while self.experience >= self.experience_to_next_level {
            self.experience -= self.experience_to_next_level;
            self.value += 1;
            self.experience_to_next_level = self.experience_to_next_level + 100;
            levels_gained += 1;
        }
        levels_gained
    }
}
