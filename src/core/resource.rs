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
use serde::de::DeserializeSeed;

use league_core::{CharacterRecord, SpellObject, UiElementEffectAnimationData, UiElementIconData};
use league_to_lol::{
    get_character_record_save_path, get_character_spell_objects_save_path, get_struct_from_file,
    CONFIG_PATH_MAP, CONFIG_PATH_MAP_NAV_GRID, CONFIG_UI,
};
use lol_config::{
    CharacterConfigsDeserializer, ConfigCharacterSkin, ConfigGame, ConfigMap, ConfigNavigationGrid,
    ConfigUi,
};
use lol_loader::{
    LeagueLoaderAnimationClip, LeagueLoaderImage, LeagueLoaderMaterial, LeagueLoaderMesh,
    LeagueLoaderMeshStatic, LeagueLoaderSkinnedMeshInverseBindposes,
};

#[derive(Default)]
pub struct PluginResource {
    pub game_config_path: String,
}

impl Plugin for PluginResource {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<LeagueLoaderMaterial>();
        app.init_asset_loader::<LeagueLoaderImage>();
        app.init_asset_loader::<LeagueLoaderMesh>();
        app.init_asset_loader::<LeagueLoaderMeshStatic>();
        app.init_asset_loader::<LeagueLoaderAnimationClip>();
        app.init_asset_loader::<LeagueLoaderSkinnedMeshInverseBindposes>();

        let mut resource_cache = ResourceCache::default();

        let config_ui: ConfigUi = get_struct_from_file(CONFIG_UI).unwrap();
        app.insert_resource(config_ui);

        let config_map: ConfigMap = get_struct_from_file(CONFIG_PATH_MAP).unwrap();

        for (_, v) in config_map.environment_objects.iter() {
            resource_cache.skins.insert(
                v.definition.skin.clone(),
                get_struct_from_file(&format!(
                    "ASSETS/{}/config_character_skin",
                    &v.definition.skin
                ))
                .unwrap(),
            );

            let character_record = get_struct_from_file::<CharacterRecord>(
                &get_character_record_save_path(&v.definition.character_record),
            )
            .unwrap();
            let spell_map = get_struct_from_file::<HashMap<u32, SpellObject>>(
                &get_character_spell_objects_save_path(&character_record.m_character_name),
            )
            .unwrap();

            resource_cache
                .character_records
                .insert(v.definition.character_record.clone(), character_record);

            resource_cache.spells.extend(spell_map);
        }

        for (_, v) in config_map.characters.iter() {
            resource_cache.skins.insert(
                v.skin.clone(),
                get_struct_from_file(&format!("ASSETS/{}/config_character_skin", &v.skin)).unwrap(),
            );
            let character_record = get_struct_from_file::<CharacterRecord>(
                &get_character_record_save_path(&v.character_record),
            )
            .unwrap();
            let spell_map = get_struct_from_file::<HashMap<u32, SpellObject>>(
                &get_character_spell_objects_save_path(&character_record.m_character_name),
            )
            .unwrap();

            resource_cache
                .character_records
                .insert(v.character_record.clone(), character_record);

            resource_cache.spells.extend(spell_map);
        }

        let mut file = File::open(format!("assets/{}", &self.game_config_path)).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        let nav_grid: ConfigNavigationGrid =
            get_struct_from_file(CONFIG_PATH_MAP_NAV_GRID).unwrap();

        app.insert_resource(config_map);
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

            let skin = get_struct_from_file::<ConfigCharacterSkin>(&format!(
                "ASSETS/{}/config_character_skin",
                &character_config.skin_path
            ))
            .unwrap();
            resource_cache
                .skins
                .insert(character_config.skin_path.clone(), skin);

            // 加载 character_record 到 resource_cache
            let character_record = get_struct_from_file::<CharacterRecord>(
                &get_character_record_save_path(&character_config.character_record),
            )
            .unwrap();
            let spell_map = get_struct_from_file::<HashMap<u32, SpellObject>>(
                &get_character_spell_objects_save_path(&character_record.m_character_name),
            )
            .unwrap();

            resource_cache
                .character_records
                .insert(character_config.character_record.clone(), character_record);

            resource_cache.spells.extend(spell_map);

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
    pub skins: HashMap<String, ConfigCharacterSkin>,
    pub character_records: HashMap<String, CharacterRecord>,
    pub spells: HashMap<u32, SpellObject>,
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
