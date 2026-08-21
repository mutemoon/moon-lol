use bevy::prelude::*;
use lol_champions::fiora::e::BuffFioraE;
use lol_champions::fiora::passive::Vital;
use lol_champions::fiora::r::BuffFioraR;
use lol_core::base::buff::Buffs;
use lol_core::base::direction::Direction;

/// Modifier 类型注册表（枚举 ID 经网络 embedding 表映射为特征向量）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u16)]
pub enum ModifierNameId {
    #[default]
    None = 0,
    FioraPassiveVital = 1,
    FioraRVitalEast = 2,
    FioraRVitalWest = 3,
    FioraRVitalNorth = 4,
    FioraRVitalSouth = 5,
    FioraBuffE = 6,
}

impl ModifierNameId {
    pub const COUNT: usize = 7;

    pub fn to_f32(self) -> f32 {
        (self as u16) as f32
    }

    pub fn from_u16(val: u16) -> Self {
        match val {
            1 => Self::FioraPassiveVital,
            2 => Self::FioraRVitalEast,
            3 => Self::FioraRVitalWest,
            4 => Self::FioraRVitalNorth,
            5 => Self::FioraRVitalSouth,
            6 => Self::FioraBuffE,
            _ => Self::None,
        }
    }
}

/// 单个 modifier 槽位的观测表示（5维：name_id, remaining_duration, stack_count, param0, param1）
/// OpenAI Five Appendix E: (remaining duration, stack count, modifier name categorical embedding)
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ModifierSlotObs {
    pub name_id: ModifierNameId,
    pub remaining_duration: f32,
    pub stack_count: f32,
    pub param0: f32, // 如相对方向 X / cos
    pub param1: f32, // 如相对方向 Z / sin
}

impl ModifierSlotObs {
    pub const DIM: usize = 5;

    pub fn to_vector(&self) -> [f32; 5] {
        [
            self.name_id.to_f32(),
            self.remaining_duration,
            self.stack_count,
            self.param0,
            self.param1,
        ]
    }
}

/// 从实体提取统一的 Modifier 槽位列表，不足 max_slots 自动以 None 槽位 0-padding
pub fn extract_entity_modifiers(
    world: &World,
    entity: Entity,
    max_slots: usize,
) -> Vec<ModifierSlotObs> {
    let mut slots = Vec::with_capacity(max_slots);

    // 1. 被动破绽
    if let Some(vital) = world.get::<Vital>(entity) {
        let (dir_x, dir_z) = match vital.direction {
            Direction::X => (1.0, 0.0),
            Direction::NegX => (-1.0, 0.0),
            Direction::Z => (0.0, 1.0),
            Direction::NegZ => (0.0, -1.0),
        };
        let is_active = vital.is_active();
        let dur = vital.remove_timer.remaining_secs();
        slots.push(ModifierSlotObs {
            name_id: ModifierNameId::FioraPassiveVital,
            remaining_duration: dur / 4.0,
            stack_count: if is_active { 1.0 } else { 0.0 },
            param0: dir_x,
            param1: dir_z,
        });
    }

    // 2. 检查 Buffs 列表
    if let Some(buffs) = world.get::<Buffs>(entity) {
        for buff_entity in buffs.iter() {
            // 大招四破绽
            if let Some(buff_r) = world.get::<BuffFioraR>(buff_entity) {
                let is_active = buff_r.is_active();
                let dur = buff_r.remove_timer.remaining_secs() / 8.0;
                let active_f = if is_active { 1.0 } else { 0.0 };

                if buff_r.vitals.contains(&Direction::X) && slots.len() < max_slots {
                    slots.push(ModifierSlotObs {
                        name_id: ModifierNameId::FioraRVitalEast,
                        remaining_duration: dur,
                        stack_count: active_f,
                        param0: 1.0,
                        param1: 0.0,
                    });
                }
                if buff_r.vitals.contains(&Direction::NegX) && slots.len() < max_slots {
                    slots.push(ModifierSlotObs {
                        name_id: ModifierNameId::FioraRVitalWest,
                        remaining_duration: dur,
                        stack_count: active_f,
                        param0: -1.0,
                        param1: 0.0,
                    });
                }
                if buff_r.vitals.contains(&Direction::Z) && slots.len() < max_slots {
                    slots.push(ModifierSlotObs {
                        name_id: ModifierNameId::FioraRVitalNorth,
                        remaining_duration: dur,
                        stack_count: active_f,
                        param0: 0.0,
                        param1: 1.0,
                    });
                }
                if buff_r.vitals.contains(&Direction::NegZ) && slots.len() < max_slots {
                    slots.push(ModifierSlotObs {
                        name_id: ModifierNameId::FioraRVitalSouth,
                        remaining_duration: dur,
                        stack_count: active_f,
                        param0: 0.0,
                        param1: -1.0,
                    });
                }
            }

            // E 技能强化
            if let Some(buff_e) = world.get::<BuffFioraE>(buff_entity) {
                if slots.len() < max_slots {
                    slots.push(ModifierSlotObs {
                        name_id: ModifierNameId::FioraBuffE,
                        remaining_duration: 0.0,
                        stack_count: buff_e.left as f32,
                        param0: 0.0,
                        param1: 0.0,
                    });
                }
            }
        }
    }

    // 0-padding 补齐固定长度
    while slots.len() < max_slots {
        slots.push(ModifierSlotObs::default());
    }

    slots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier_slot_padding_and_encoding() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // 无任何 buff 时，提取 4 个槽位应全部为 None (0-padding)
        let slots = extract_entity_modifiers(&world, entity, 4);
        assert_eq!(slots.len(), 4);
        for slot in &slots {
            assert_eq!(slot.name_id, ModifierNameId::None);
            assert_eq!(slot.remaining_duration, 0.0);
            assert_eq!(slot.stack_count, 0.0);
            let vec = slot.to_vector();
            assert_eq!(vec, [0.0, 0.0, 0.0, 0.0, 0.0]);
        }
    }

    #[test]
    fn test_modifier_passive_vital_extraction() {
        let mut world = World::new();
        let entity = world
            .spawn(Vital::new(Direction::X, 4.0, 1.7))
            .id();

        let slots = extract_entity_modifiers(&world, entity, 4);
        assert_eq!(slots.len(), 4);
        assert_eq!(slots[0].name_id, ModifierNameId::FioraPassiveVital);
        assert_eq!(slots[0].param0, 1.0); // +X
        assert_eq!(slots[0].param1, 0.0);

        // 其余 3 个槽位应为 0-padding
        for slot in &slots[1..] {
            assert_eq!(slot.name_id, ModifierNameId::None);
        }
    }
}

