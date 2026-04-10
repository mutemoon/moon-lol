use bevy::prelude::*;
use league_utils::hash_wad;
use lol_base::grid::ConfigNavigationGrid;

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ResourceGrid(pub Handle<ConfigNavigationGrid>);

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ResourceGridPath(pub String);

pub fn update_load_grid(
    mut commands: Commands,
    res_asset_server: Res<AssetServer>,
    res_assets_map_container: Res<ResourceGridPath>,
) {
    commands.insert_resource(ResourceGrid(res_asset_server.load(format!(
        "maps/{:x}.nav_grid",
        hash_wad(&res_assets_map_container.0)
    ))));
}
