use bevy::prelude::*;

use crate::{Level, SkillPoints, State};

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(State, Level = Level { value: 1, experience: 0, experience_to_next_level: 280 }, SkillPoints)]
pub struct Champion;

#[derive(Default)]
pub struct PluginChampion;

impl Plugin for PluginChampion {
    fn build(&self, _app: &mut App) {}
}
