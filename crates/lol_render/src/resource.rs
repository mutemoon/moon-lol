use std::collections::HashMap;

use bevy::prelude::*;
use league_core::extract::{AnimationGraphData, SkinCharacterDataProperties};
use league_file::skeleton::LeagueSkeleton;
use lol_base::prop::HashKey;
use lol_core::resource::loading::RegisterLoadingExt;
use lol_core::utils::AssetServerLoadLeague;

use crate::loaders::animation::LeagueLoaderAnimationClip;
use crate::loaders::image::LeagueLoaderImage;
use crate::loaders::mesh::LeagueLoaderMesh;
use crate::loaders::mesh_static::LeagueLoaderMeshStatic;
use crate::loaders::shader::LeagueLoaderShaderToc;
use crate::loaders::skeleton::LeagueLoaderSkeleton;
use crate::shader::{startup_load_shaders, update_shaders, ResourceShaderHandles};
use crate::skin::LeagueSkinMesh;

#[derive(Default)]
pub struct PluginRenderResource;

impl Plugin for PluginRenderResource {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<LeagueLoaderImage>();
        app.init_asset_loader::<LeagueLoaderMesh>();
        app.init_asset_loader::<LeagueLoaderSkeleton>();
        app.init_asset_loader::<LeagueLoaderMeshStatic>();
        app.init_asset_loader::<LeagueLoaderAnimationClip>();
        app.init_asset_loader::<LeagueLoaderShaderToc>();

        app.register_loading::<HashKey<AnimationGraphData>>()
            .register_loading::<HashKey<SkinCharacterDataProperties>>()
            .register_loading::<Handle<LeagueSkeleton>>()
            .register_loading::<(Handle<LeagueSkinMesh>, Handle<StandardMaterial>)>();

        app.init_resource::<ResourceCache>();
        app.init_resource::<ResourceShaderHandles>();

        app.add_systems(Startup, startup_load_shaders);
        app.add_systems(Update, update_shaders);
    }
}

#[derive(Resource, Default)]
pub struct ResourceCache {
    image: HashMap<String, Handle<Image>>,
    mesh: HashMap<String, Handle<Mesh>>,
}

impl ResourceCache {
    pub fn get_image(&mut self, asset_server: &AssetServer, path: &str) -> Handle<Image> {
        match self.image.get(path) {
            Some(handle) => handle.clone(),
            None => {
                let handle = asset_server.load_league(path);
                self.image.insert(path.to_string(), handle.clone());
                handle
            }
        }
    }

    pub fn get_image_srgb(&mut self, asset_server: &AssetServer, path: &str) -> Handle<Image> {
        match self.image.get(path) {
            Some(handle) => handle.clone(),
            None => {
                let handle = asset_server.load_league_labeled(path, "srgb");
                self.image.insert(path.to_string(), handle.clone());
                handle
            }
        }
    }

    pub fn get_mesh(&mut self, asset_server: &AssetServer, path: &str) -> Handle<Mesh> {
        match self.mesh.get(path) {
            Some(handle) => handle.clone(),
            None => {
                let handle = asset_server.load_league(path);
                self.mesh.insert(path.to_string(), handle.clone());
                handle
            }
        }
    }
}
