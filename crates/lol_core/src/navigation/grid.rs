use bevy::prelude::*;
use lol_base::grid::ConfigNavigationGrid;

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ResourceGrid(pub Handle<ConfigNavigationGrid>);
