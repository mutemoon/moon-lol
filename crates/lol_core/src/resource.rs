pub mod loading;
pub mod prop_bin;

use std::fs::File;
use std::io::Read;

use bevy::ecs::component::ComponentCloneBehavior;
use bevy::ecs::entity::{EntityHashMap, SceneEntityMapper};
use bevy::ecs::relationship::RelationshipHookMode;
use bevy::prelude::*;
use bevy::scene::ron::{self};
use lol_base::game::{CharacterConfigsDeserializer, ConfigGame};
use lol_base::grid::ConfigNavigationGrid;
use lol_base::mapgeo::ConfigMapGeo;
use lol_base::register::init_league_asset;
use lol_base::shader::{ResourceShaderChunk, ResourceShaderPackage};
use lol_loader::mapgeo::LeagueLoaderMapgeo;
use lol_loader::navgrid::LeagueLoaderNavGrid;
use lol_loader::property::LeagueLoaderProperty;
use serde::de::DeserializeSeed;

use self::loading::PluginResourceLoading;
use self::prop_bin::PluginResourcePropBin;

#[derive(Default)]
pub struct PluginResource {
    pub game_config_path: String,
}

impl Plugin for PluginResource {
    fn build(&self, app: &mut App) {
        app.init_asset::<ConfigMapGeo>();
        app.init_asset::<ResourceShaderPackage>();
        app.init_asset::<ResourceShaderChunk>();
        app.init_asset::<ConfigNavigationGrid>();

        app.init_asset_loader::<LeagueLoaderProperty>();
        app.init_asset_loader::<LeagueLoaderMapgeo>();
        app.init_asset_loader::<LeagueLoaderNavGrid>();

        init_league_asset(app);

        app.add_plugins(PluginResourceLoading);
        app.add_plugins(PluginResourcePropBin);

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
