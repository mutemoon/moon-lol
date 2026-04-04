use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::loaders::animation::LeagueLoaderAnimationClip;
use crate::loaders::image::LeagueLoaderImage;
use crate::loaders::mesh::LeagueLoaderMesh;
use crate::loaders::mesh_static::LeagueLoaderMeshStatic;
use crate::loaders::shader::LeagueLoaderShaderToc;
use crate::loaders::skeleton::LeagueLoaderSkeleton;
use crate::skin::LeagueSkinMesh;
use bevy::ecs::component::ComponentCloneBehavior;
use bevy::ecs::entity::{EntityHashMap, SceneEntityMapper};
use bevy::ecs::relationship::RelationshipHookMode;
use bevy::prelude::*;
use bevy::scene::ron::{self};
use league_core::extract::{AnimationGraphData, SkinCharacterDataProperties};
use league_file::skeleton::LeagueSkeleton;
use lol_config::game::{CharacterConfigsDeserializer, ConfigGame};
use lol_config::grid::ConfigNavigationGrid;
use lol_config::mapgeo::ConfigMapGeo;
use lol_config::prop::HashKey;
use lol_config::register::init_league_asset;
use lol_config::shader::{ResourceShaderChunk, ResourceShaderPackage};
use lol_core::resource::loading::RegisterLoadingExt;
use lol_core::utils::AssetServerLoadLeague;
use serde::de::DeserializeSeed;

use crate::shader::{startup_load_shaders, update_shaders, ResourceShaderHandles};

#[derive(Default)]
pub struct PluginResource {
    pub game_config_path: String,
}

impl Plugin for PluginResource {
    fn build(&self, app: &mut App) {
        app.init_asset::<LeagueSkinMesh>();
        app.init_asset::<LeagueSkeleton>();

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
