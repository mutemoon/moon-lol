pub mod loading;
pub mod prop_bin;
pub mod shader;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use bevy::ecs::component::ComponentCloneBehavior;
use bevy::ecs::entity::{EntityHashMap, SceneEntityMapper};
use bevy::ecs::relationship::RelationshipHookMode;
use bevy::prelude::*;
use bevy::scene::ron::{self};
use league_file::skeleton::LeagueSkeleton;
use lol_config::game::{CharacterConfigsDeserializer, ConfigGame};
use lol_config::grid::ConfigNavigationGrid;
use lol_config::mapgeo::ConfigMapGeo;
use lol_config::register::init_league_asset;
use lol_config::shader::{ResourceShaderChunk, ResourceShaderPackage};
use lol_core::skin::LeagueSkinMesh;
use lol_core_render::utils::AssetServerLoadLeague;
use lol_loader::animation::LeagueLoaderAnimationClip;
use lol_loader::image::LeagueLoaderImage;
use lol_loader::mapgeo::LeagueLoaderMapgeo;
use lol_loader::mesh::LeagueLoaderMesh;
use lol_loader::mesh_static::LeagueLoaderMeshStatic;
use lol_loader::navgrid::LeagueLoaderNavGrid;
use lol_loader::property::LeagueLoaderProperty;
use lol_loader::shader::LeagueLoaderShaderToc;
use lol_loader::skeleton::LeagueLoaderSkeleton;
use serde::de::DeserializeSeed;

use self::loading::PluginResourceLoading;
use self::prop_bin::PluginResourcePropBin;
use self::shader::{startup_load_shaders, update_shaders, ResourceShaderHandles};

#[derive(Default)]
pub struct PluginResource {
    pub game_config_path: String,
}

impl Plugin for PluginResource {
    fn build(&self, app: &mut App) {
        app.init_asset::<ConfigMapGeo>();
        app.init_asset::<LeagueSkeleton>();
        app.init_asset::<LeagueSkinMesh>();
        app.init_asset::<ResourceShaderPackage>();
        app.init_asset::<ResourceShaderChunk>();
        app.init_asset::<ConfigNavigationGrid>();

        app.init_asset_loader::<LeagueLoaderProperty>();
        app.init_asset_loader::<LeagueLoaderImage>();
        app.init_asset_loader::<LeagueLoaderMesh>();
        app.init_asset_loader::<LeagueLoaderSkeleton>();
        app.init_asset_loader::<LeagueLoaderMapgeo>();
        app.init_asset_loader::<LeagueLoaderMeshStatic>();
        app.init_asset_loader::<LeagueLoaderAnimationClip>();
        app.init_asset_loader::<LeagueLoaderShaderToc>();
        app.init_asset_loader::<LeagueLoaderNavGrid>();

        init_league_asset(app);

        app.init_resource::<ResourceShaderHandles>();
        app.init_resource::<ResourceCache>();

        app.add_plugins(PluginResourceLoading);
        app.add_plugins(PluginResourcePropBin);

        app.add_systems(Startup, startup_load_shaders);
        app.add_systems(Update, update_shaders);

        let mut file = File::open(format!("assets/{}", &self.game_config_path)).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        let world = app.world_mut();

        let type_registry = world.resource::<AppTypeRegistry>();
        // println!("{:?}", type_registry.0);

        let binding = type_registry.internal.clone();
        let type_registry = binding.read().unwrap();

        let mut deserializer = ron::de::Deserializer::from_bytes(&data).unwrap();
        let scene_deserializer = CharacterConfigsDeserializer {
            type_registry: &type_registry,
        };
        let config_legends = scene_deserializer.deserialize(&mut deserializer).unwrap();

        let mut legends = Vec::new();
        for (character_config, components) in config_legends {
            let entity = world.spawn_empty().id();

            legends.push((
                entity,
                character_config.skin_path.clone(),
                character_config.character_record.clone(),
            ));

            for component in &components {
                let type_info = component.get_represented_type_info().unwrap();
                let registration = type_registry.get(type_info.type_id()).unwrap();
                let reflect_component = registration.data::<ReflectComponent>().unwrap();

                let component_id = reflect_component.register_component(world);

                #[expect(unsafe_code, reason = "this is faster")]
                let component_info = unsafe { world.components().get_info_unchecked(component_id) };
                if matches!(
                    *component_info.clone_behavior(),
                    ComponentCloneBehavior::Ignore
                ) {
                    continue;
                }

                SceneEntityMapper::world_scope(
                    &mut EntityHashMap::new(),
                    world,
                    |world, mapper| {
                        reflect_component.apply_or_insert_mapped(
                            &mut world.entity_mut(entity),
                            component.as_partial_reflect(),
                            &type_registry,
                            mapper,
                            RelationshipHookMode::Skip,
                        );
                    },
                );
            }
        }

        app.insert_resource(ConfigGame { legends });
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
