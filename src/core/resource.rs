use std::fs::File;
use std::io::Read;

use bevy::ecs::component::ComponentCloneBehavior;
use bevy::ecs::entity::{EntityHashMap, SceneEntityMapper};
use bevy::ecs::relationship::RelationshipHookMode;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::scene::ron::{self};
use league_core::init_league_asset;
use league_file::{LeagueMapGeo, LeagueSkeleton};
use league_to_lol::{get_struct_from_file, CONFIG_PATH_MAP_NAV_GRID};
use lol_config::{CharacterConfigsDeserializer, ConfigGame, ConfigNavigationGrid, LeagueProperty};
use lol_loader::{
    LeagueLoaderAnimationClip, LeagueLoaderAny, LeagueLoaderImage, LeagueLoaderMesh,
    LeagueLoaderMeshStatic, LeagueLoaderProperty,
};
use serde::de::DeserializeSeed;

#[derive(Default)]
pub struct PluginResource {
    pub game_config_path: String,
}

impl Plugin for PluginResource {
    fn build(&self, app: &mut App) {
        app.init_asset::<LeagueMapGeo>();
        app.init_asset::<LeagueSkeleton>();
        app.init_asset::<LeagueProperty>();

        init_league_asset(app);

        app.init_asset_loader::<LeagueLoaderAny>();
        app.init_asset_loader::<LeagueLoaderProperty>();
        app.init_asset_loader::<LeagueLoaderImage>();
        app.init_asset_loader::<LeagueLoaderMesh>();
        app.init_asset_loader::<LeagueLoaderMeshStatic>();
        app.init_asset_loader::<LeagueLoaderAnimationClip>();

        app.add_systems(Startup, startup_load_prop_bin);

        let mut resource_cache = ResourceCache::default();

        let mut file = File::open(format!("assets/{}", &self.game_config_path)).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        let nav_grid: ConfigNavigationGrid =
            get_struct_from_file(CONFIG_PATH_MAP_NAV_GRID).unwrap();

        app.insert_resource(nav_grid);

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

        // for (_, ui_element) in &resource_cache.ui_elements {
        //     if let Some(texture_data) = ui_element.texture_data.as_ref() {
        //         if let EnumData::AtlasData(atlas_data) = texture_data
        //         {
        //             // if atlas_data.m_texture_name.contains("Clarity_HUDAtlas") {
        //             if ui_element.name.contains("PlayerFrame") {
        //                 println!("{:?}", ui_element.name);
        //                 println!("{:?}", atlas_data.m_texture_uv);
        //             }
        //         }
        //     }
        // }
        app.insert_resource(resource_cache);
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
                let handle = asset_server.load(path.to_string());
                self.image.insert(path.to_string(), handle.clone());
                handle
            }
        }
    }

    pub fn get_mesh(&mut self, asset_server: &AssetServer, path: &str) -> Handle<Mesh> {
        match self.mesh.get(path) {
            Some(handle) => handle.clone(),
            None => {
                let handle = asset_server.load(path.to_string());
                self.mesh.insert(path.to_string(), handle.clone());
                handle
            }
        }
    }
}

fn startup_load_prop_bin(res_asset_server: Res<AssetServer>) {
    let prop_bin_paths = vec![
        "data/maps/mapgeometry/map11/base_srx.materials.bin",
        "data/maps/shipping/map11/map11.bin",
    ];

    for path in prop_bin_paths {
        res_asset_server.load_untyped(path);
    }
}
