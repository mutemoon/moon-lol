use bevy::prelude::{Entity, EntityEvent, Resource, Vec2};

use super::enums::SkillSlot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillCastFailureReason {
    MissingSkills,
    InvalidSkillIndex,
    MissingSkillEntity,
    MissingSpellObject,
    NotLearned,
    MissingAbilityResource,
    InsufficientAbilityResource,
    CoolingDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillCastResult {
    Started,
    Failed(SkillCastFailureReason),
}

#[derive(Debug, Clone)]
pub struct SkillCastRecord {
    pub caster: Entity,
    pub skill_entity: Option<Entity>,
    pub index: usize,
    pub slot: Option<SkillSlot>,
    pub point: Vec2,
    pub result: SkillCastResult,
}

#[derive(Resource, Default, Debug)]
pub struct SkillCastLog(pub Vec<SkillCastRecord>);

#[derive(EntityEvent)]
pub struct CommandSkillStart {
    pub entity: Entity,
    pub index: usize,
    pub point: Vec2,
}

#[derive(EntityEvent)]
pub struct CommandSkillBeforeStart {
    pub entity: Entity,
    pub index: usize,
    pub point: Vec2,
}

#[derive(EntityEvent, Debug, Clone, Copy)]
pub struct EventSkillCast {
    pub entity: Entity,
    pub skill_entity: Entity,
    pub index: usize,
    pub point: Vec2,
}

#[derive(EntityEvent)]
pub struct CommandSkillLevelUp {
    pub entity: Entity,
    pub index: usize,
}
