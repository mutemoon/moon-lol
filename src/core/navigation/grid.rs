use bevy::prelude::*;
use league_core::{EnumMap, MapContainer};
use lol_config::{ConfigNavigationGrid, LoadHashKeyTrait};

use crate::{AssetServerLoadLeague, MapName};

#[derive(Resource, Default)]
pub struct ResourceGrid(pub Handle<ConfigNavigationGrid>);

pub fn update_load_grid(
    mut commands: Commands,
    res_asset_server: Res<AssetServer>,
    res_assets_map_container: Res<Assets<MapContainer>>,
    res_map_name: Res<MapName>,
) {
    let map_container = res_assets_map_container
        .load_hash(&res_map_name.get_materials_path())
        .unwrap();

    for item in &map_container.components {
        let EnumMap::MapNavGrid(map_nav_grid) = item else {
            continue;
        };

        commands.insert_resource(ResourceGrid(
            res_asset_server.load_league::<ConfigNavigationGrid>(&map_nav_grid.nav_grid_path),
        ));
    }
}
