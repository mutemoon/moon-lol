use std::fs::File;

use crate::{
    config::Configs,
    league::{LeagueLoaderAnimation, LeagueLoaderImage, LeagueLoaderMaterial, LeagueLoaderMesh},
};
use bevy::{prelude::*, scene::ron::de::from_reader};

pub struct PluginResource;

impl Plugin for PluginResource {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<LeagueLoaderMaterial>();
        app.init_asset_loader::<LeagueLoaderImage>();
        app.init_asset_loader::<LeagueLoaderMesh>();
        app.init_asset_loader::<LeagueLoaderAnimation>();

        let configs: Configs = from_reader(File::open("assets/configs.ron").unwrap()).unwrap();
        app.insert_resource(configs);
    }
}
