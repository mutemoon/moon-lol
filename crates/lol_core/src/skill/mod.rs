mod enums;
mod events;
mod helpers;
mod observers;
#[cfg(test)]
mod tests;

use std::ops::Deref;

use bevy::prelude::{Component, Handle, Reflect, ReflectComponent, *};
pub use enums::*;
pub use events::*;
pub use helpers::*;
use lol_base::spell::Spell;
pub use observers::*;

use crate::loaders::spell::LoaderSpell;

#[derive(Default)]
pub struct PluginSkill;

impl Plugin for PluginSkill {
    fn build(&self, app: &mut App) {
        app.init_asset::<lol_base::spell::Spell>();
        app.init_asset_loader::<LoaderSpell>();

        app.init_resource::<SkillCastLog>();

        app.add_observer(on_skill_cast);
        app.add_observer(on_skill_level_up);
        app.add_observer(on_level_up);
        app.add_systems(FixedUpdate, update_skill_recast_windows);
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Skill {
    pub spell: Handle<Spell>,
    pub level: usize,
    pub slot: SkillSlot,
    pub cooldown_mode: SkillCooldownMode,
}

impl Default for Skill {
    fn default() -> Self {
        Self {
            spell: Handle::<Spell>::default(),
            level: 0,
            slot: SkillSlot::Q,
            cooldown_mode: SkillCooldownMode::AfterCast,
        }
    }
}

impl Skill {
    pub fn new(slot: SkillSlot, spell: Handle<Spell>) -> Self {
        Self {
            spell,
            level: 0,
            slot,
            cooldown_mode: SkillCooldownMode::AfterCast,
        }
    }

    pub fn with_level(mut self, level: usize) -> Self {
        self.level = level;
        self
    }

    pub fn with_cooldown_mode(mut self, cooldown_mode: SkillCooldownMode) -> Self {
        self.cooldown_mode = cooldown_mode;
        self
    }
}

#[derive(Component, Reflect, Debug)]
#[relationship(relationship_target = Skills)]
#[reflect(Component)]
pub struct SkillOf(pub Entity);

#[derive(Component, Reflect, Debug)]
#[relationship_target(relationship = SkillOf, linked_spawn)]
#[reflect(Component)]
pub struct Skills(Vec<Entity>);

#[derive(Component, Debug)]
#[relationship(relationship_target = PassiveSkill)]
pub struct PassiveSkillOf(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = PassiveSkillOf, linked_spawn)]
pub struct PassiveSkill(Entity);

impl Deref for Skills {
    type Target = Vec<Entity>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Skills {
    fn default() -> Self {
        Skills(Vec::new())
    }
}

impl Skills {
    /// Create a new Skills list with a single skill entity
    pub fn new(entity: Entity) -> Self {
        Skills(vec![entity])
    }

    /// Create a Skills list from a vector of entities
    pub fn from_vec(entities: Vec<Entity>) -> Self {
        Skills(entities)
    }

    /// Add a skill entity to this Skills list
    pub fn push(&mut self, entity: Entity) {
        self.0.push(entity);
    }

    /// Get a skill entity by index
    pub fn get(&self, index: usize) -> Option<&Entity> {
        self.0.get(index)
    }
}

/// 技能冷却状态（运行时状态，包含 timer）
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct CoolDown {
    pub timer: Option<Timer>,
    pub duration: f32,
}

#[derive(Component)]
pub struct SkillPoints(pub u32);

impl Default for SkillPoints {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Component, Debug, Clone)]
pub struct SkillRecastWindow {
    pub stage: u8,
    pub max_stage: u8,
    pub timer: Timer,
}

impl SkillRecastWindow {
    pub fn new(stage: u8, max_stage: u8, duration: f32) -> Self {
        Self {
            stage,
            max_stage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}
