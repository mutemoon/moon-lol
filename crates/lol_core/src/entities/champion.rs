use bevy::prelude::*;

use crate::base::level::Level;
use crate::base::state::State;
use crate::character::Character;
use crate::skill::SkillPoints;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(State, Character, Level = Level { value: 1, experience: 0, experience_to_next_level: 280 }, SkillPoints)]
pub struct Champion;

#[derive(Default)]
pub struct PluginChampion;

impl Plugin for PluginChampion {
    fn build(&self, _app: &mut App) {}
}
