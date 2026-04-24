use std::collections::HashMap;
use std::f32;
use std::fmt::Display;

use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use lol_loader::barrack::BarracksLoader;
use lol_loader::navgrid::NavGridLoader;

use crate::lane::Lane;

pub const MAP_WIDTH: f32 = 14400.0;
pub const MAP_HEIGHT: f32 = 14765.0;

pub const MAP_OFFSET_X: f32 = 300.0;
pub const MAP_OFFSET_Y: f32 = 520.0;

#[derive(Default)]
pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<BarracksLoader>();
        app.init_asset_loader::<NavGridLoader>();

        app.init_resource::<MapName>();
        app.init_resource::<MinionPath>();

        app.init_state::<MapState>();

        app.add_systems(Startup, startup_load_map_geometry);
        app.add_observer(update_load_map_geometry);
    }
}

#[derive(States, Default, Debug, Hash, Eq, Clone, PartialEq)]
pub enum MapState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Component)]
pub struct MapGeometry {
    pub bounding_box: Aabb3d,
}

#[derive(Resource)]
pub struct MapName(pub String);

impl Default for MapName {
    fn default() -> Self {
        Self("sr_seasonal_map".to_string())
    }
}

impl MapName {
    pub fn get_materials_path(&self) -> String {
        format!("Maps/MapGeometry/Map11/{}", &self.0)
    }

    pub fn get_materials_bin_path(&self) -> String {
        format!("data/{}.materials.bin", &self.get_materials_path())
    }

    pub fn get_ron_path(&self) -> String {
        format!("maps/{}.ron", &self.0)
    }

    pub fn get_mapgeo_path(&self) -> String {
        format!("data/maps/mapgeometry/map11/{}.mapgeo", &self.0)
    }
}

impl Display for MapName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct MinionPath(pub HashMap<Lane, Vec<Vec2>>);

#[derive(Resource)]
pub struct DynamicWorldHandle(pub Handle<DynamicWorld>);

fn startup_load_map_geometry(
    mut commands: Commands,
    res_map_name: Res<MapName>,
    res_asset_server: Res<AssetServer>,
) {
    commands.spawn(DynamicWorldRoot(
        res_asset_server.load(res_map_name.get_ron_path()),
    ));
}

fn update_load_map_geometry(_trigger: On<WorldInstanceReady>, mut commands: Commands) {
    commands.set_state(MapState::Loaded);
}
