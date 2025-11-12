use bevy::prelude::*;

use crate::State;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(State)]
pub struct Champion;

#[derive(Default)]
pub struct PluginChampion;

impl Plugin for PluginChampion {
    fn build(&self, app: &mut App) {
        app.register_type::<Champion>();
    }
}
