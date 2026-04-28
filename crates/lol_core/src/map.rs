use std::collections::BTreeMap;
use std::f32;

use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;
use lol_base::map::MapPaths;

use crate::lane::Lane;

pub const MAP_WIDTH: f32 = 14400.0;
pub const MAP_HEIGHT: f32 = 14765.0;

pub const MAP_OFFSET_X: f32 = 300.0;
pub const MAP_OFFSET_Y: f32 = 520.0;

#[derive(Default)]
pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapPaths>();
        app.init_resource::<MinionPath>();

        app.add_systems(Startup, startup_load_map_geometry);
    }
}

#[derive(Component)]
pub struct MapGeometry {
    pub bounding_box: Aabb3d,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct MinionPath(pub BTreeMap<Lane, Vec<Vec2>>);

#[derive(Resource)]
pub struct DynamicWorldHandle(pub Handle<DynamicWorld>);

fn startup_load_map_geometry(
    mut commands: Commands,
    res_map_paths: Res<MapPaths>,
    res_asset_server: Res<AssetServer>,
) {
    commands.spawn(DynamicWorldRoot(
        res_asset_server.load(res_map_paths.scene_ron()),
    ));
}
