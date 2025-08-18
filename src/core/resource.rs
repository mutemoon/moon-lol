use crate::{
    core::{ConfigGame, ConfigMap, ConfigNavigationGrid},
    league::{
        get_struct_from_file, LeagueLoaderAnimationClip, LeagueLoaderImage, LeagueLoaderMaterial,
        LeagueLoaderMesh, LeagueLoaderSkinnedMeshInverseBindposes,
    },
};

pub const CONFIG_PATH_MAP: &str = "config_map";
pub const CONFIG_PATH_MAP_NAV_GRID: &str = "config_map_nav_grid";
pub const CONFIG_PATH_GAME: &str = "config_game";

use bevy::prelude::*;

#[derive(Default)]
pub struct PluginResource;

impl Plugin for PluginResource {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<LeagueLoaderMaterial>();
        app.init_asset_loader::<LeagueLoaderImage>();
        app.init_asset_loader::<LeagueLoaderMesh>();
        app.init_asset_loader::<LeagueLoaderAnimationClip>();
        app.init_asset_loader::<LeagueLoaderSkinnedMeshInverseBindposes>();

        let configs: ConfigMap = get_struct_from_file(CONFIG_PATH_MAP).unwrap();
        app.insert_resource(configs);

        let game_configs: ConfigGame = get_struct_from_file(CONFIG_PATH_GAME).unwrap();
        app.insert_resource(game_configs);

        let nav_grid: ConfigNavigationGrid =
            get_struct_from_file(CONFIG_PATH_MAP_NAV_GRID).unwrap();
        app.insert_resource(nav_grid);
    }
}
