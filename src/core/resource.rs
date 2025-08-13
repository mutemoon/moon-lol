use crate::{
    core::Configs,
    league::{
        get_struct_from_file, LeagueLoaderAnimationClip, LeagueLoaderImage, LeagueLoaderMaterial,
        LeagueLoaderMesh, LeagueLoaderSkinnedMeshInverseBindposes,
    },
};

use bevy::prelude::*;

pub struct PluginResource;

impl Plugin for PluginResource {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<LeagueLoaderMaterial>();
        app.init_asset_loader::<LeagueLoaderImage>();
        app.init_asset_loader::<LeagueLoaderMesh>();
        app.init_asset_loader::<LeagueLoaderAnimationClip>();
        app.init_asset_loader::<LeagueLoaderSkinnedMeshInverseBindposes>();

        let configs: Configs = get_struct_from_file("configs").unwrap();
        app.insert_resource(configs);
    }
}
