use bevy::prelude::*;
use vleue_navigator::NavMesh;

pub const MAP_WIDTH: f32 = 17000.0;
pub const MAP_HEIGHT: f32 = 17000.0;

pub const MAP_OFFSET_X: f32 = 500.0;
pub const MAP_OFFSET_Y: f32 = 500.0;

#[derive(Component)]
#[require(Visibility)]
pub struct Map;

#[derive(Resource)]
pub struct LoadedNavMesh(pub NavMesh);
