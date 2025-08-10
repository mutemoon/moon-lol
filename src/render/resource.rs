use crate::league::{LeagueLoaderImage, LeagueLoaderMaterial, LeagueLoaderMesh};
use bevy::prelude::*;

pub struct PluginResource;

impl Plugin for PluginResource {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<LeagueLoaderMaterial>();
        app.init_asset_loader::<LeagueLoaderImage>();
        app.init_asset_loader::<LeagueLoaderMesh>();
    }
}
