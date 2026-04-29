use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default, Serialize, Deserialize)]
pub enum SkillSlot {
    Passive,
    #[default]
    Q,
    W,
    E,
    R,
    Custom(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default, Serialize, Deserialize)]
pub enum SkillCooldownMode {
    #[default]
    AfterCast,
    Manual,
}

pub fn skill_slot_from_index(index: usize) -> SkillSlot {
    match index {
        0 => SkillSlot::Q,
        1 => SkillSlot::W,
        2 => SkillSlot::E,
        3 => SkillSlot::R,
        other => SkillSlot::Custom(other as u8),
    }
}
