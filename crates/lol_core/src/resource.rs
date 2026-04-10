pub mod loading;
pub mod prop_bin;

use bevy::prelude::*;
use lol_base::game::ConfigGame;
use lol_base::grid::ConfigNavigationGrid;
use lol_base::register::init_league_asset;
use lol_base::shader::{ResourceShaderChunk, ResourceShaderPackage};
use lol_loader::mapgeo::LeagueLoaderMapgeo;
use lol_loader::property::LeagueLoaderProperty;

use self::loading::PluginResourceLoading;
use self::prop_bin::PluginResourcePropBin;

#[derive(Default)]
pub struct PluginResource {
    pub game_config_path: String,
}

impl Plugin for PluginResource {
    fn build(&self, app: &mut App) {
        app.init_asset::<ResourceShaderPackage>();
        app.init_asset::<ResourceShaderChunk>();
        app.init_asset::<ConfigNavigationGrid>();

        app.init_asset_loader::<LeagueLoaderProperty>();
        app.init_asset_loader::<LeagueLoaderMapgeo>();

        init_league_asset(app);

        app.add_plugins(PluginResourceLoading);
        app.add_plugins(PluginResourcePropBin);

        app.insert_resource(ConfigGame {});
    }
}
