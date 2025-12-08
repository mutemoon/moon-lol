use std::{fs::File, io::Read};

use bevy::{
    ecs::{
        component::ComponentCloneBehavior,
        entity::{EntityHashMap, SceneEntityMapper},
        relationship::RelationshipHookMode,
    },
    platform::collections::HashMap,
    prelude::*,
    scene::ron::{self},
};
use league_property::{init_league_asset, AssetLoaderRegistry};
use serde::de::DeserializeSeed;

use league_core::{CharacterRecord, SpellObject};
use league_to_lol::{get_struct_from_file, CONFIG_PATH_MAP_NAV_GRID, CONFIG_UI};
use lol_config::{CharacterConfigsDeserializer, ConfigGame, ConfigNavigationGrid, ConfigUi};
use lol_loader::{
    LeagueLoaderAnimationClip, LeagueLoaderImage, LeagueLoaderMesh, LeagueLoaderMeshStatic,
};

use crate::SkillEffect;

#[derive(Default)]
pub struct PluginResource {
    pub game_config_path: String,
}

impl Plugin for PluginResource {
    fn build(&self, app: &mut App) {
        app.init_asset::<SpellObject>();
        app.init_asset::<SkillEffect>();
        app.init_asset::<CharacterRecord>();

        let mut asset_loader_registry = AssetLoaderRegistry::default();
        init_league_asset(&mut asset_loader_registry);

        let prop_bin_paths = vec![
            "data/maps/mapgeometry/map11/base_srx.materials.bin",
            "data/maps/shipping/map11/map11.bin",
        ];

        //     world.resource_scope(|world, registry: Mut<AssetLoaderRegistry>| {
        //         for (type_name, json_content) in raw_data {
        //             if let Some(loader) = registry.loaders.get(&type_name) {
        //                 // 调用动态分发的函数
        //                 loader.load_and_add(world, &json_content);
        //             } else {
        //                 warn!("未知的资产类型: {}", type_name);
        //             }
        //         }
        //     });

        app.init_asset_loader::<LeagueLoaderImage>();
        app.init_asset_loader::<LeagueLoaderMesh>();
        app.init_asset_loader::<LeagueLoaderMeshStatic>();
        app.init_asset_loader::<LeagueLoaderAnimationClip>();

        let mut resource_cache = ResourceCache::default();

        let config_ui: ConfigUi = get_struct_from_file(CONFIG_UI).unwrap();
        app.insert_resource(config_ui);

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
        // println!("{:?}", type_registry.iter().collect::<Vec<_>>());
        for item in type_registry.iter() {
            println!("{:?}", item.type_info().type_path_table().short_path());
        }

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
        //         if let UiElementEffectAnimationDataTextureData::AtlasData(atlas_data) = texture_data
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
