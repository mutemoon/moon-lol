mod shader;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;

use bevy::ecs::component::ComponentCloneBehavior;
use bevy::ecs::entity::{EntityHashMap, SceneEntityMapper};
use bevy::ecs::relationship::RelationshipHookMode;
use bevy::prelude::*;
use bevy::scene::ron::{self};
use league_file::LeagueSkeleton;
use league_to_lol::{get_struct_from_file, CONFIG_PATH_MAP_NAV_GRID};
use lol_config::{
    init_league_asset, CharacterConfigsDeserializer, ConfigGame, ConfigMapGeo,
    ConfigNavigationGrid, LeagueProperties, ResourceShaderChunk, ResourceShaderPackage,
    ASSET_LOADER_REGISTRY,
};
use lol_core::LeagueSkinMesh;
use lol_loader::{
    ImageSettings, LeagueLoaderAnimationClip, LeagueLoaderImage, LeagueLoaderMapgeo,
    LeagueLoaderMesh, LeagueLoaderMeshStatic, LeagueLoaderProperty, LeagueLoaderShaderToc,
    LeagueLoaderSkeleton,
};
use serde::de::DeserializeSeed;
pub use shader::*;

use crate::{
    AssetServerLoadLeague, CharacterSpawn, SkinAnimationSpawn, SkinMeshSpawn, SkinSkeletonSpawn,
    SkinSpawn,
};

#[derive(Default)]
pub struct PluginResource {
    pub game_config_path: String,
}

impl Plugin for PluginResource {
    fn build(&self, app: &mut App) {
        app.init_asset::<ConfigMapGeo>();
        app.init_asset::<LeagueSkeleton>();
        app.init_asset::<LeagueProperties>();
        app.init_asset::<LeagueSkinMesh>();
        app.init_asset::<ResourceShaderPackage>();
        app.init_asset::<ResourceShaderChunk>();

        init_league_asset(app);

        app.init_asset_loader::<LeagueLoaderProperty>();
        app.init_asset_loader::<LeagueLoaderImage>();
        app.init_asset_loader::<LeagueLoaderMesh>();
        app.init_asset_loader::<LeagueLoaderSkeleton>();
        app.init_asset_loader::<LeagueLoaderMapgeo>();
        app.init_asset_loader::<LeagueLoaderMeshStatic>();
        app.init_asset_loader::<LeagueLoaderAnimationClip>();
        app.init_asset_loader::<LeagueLoaderShaderToc>();

        app.init_resource::<ResourceShaderHandles>();
        app.init_resource::<LeagueProperties>();
        app.init_resource::<LeaguePropertyFiles>();

        app.add_systems(Startup, startup_load_shaders);
        app.add_systems(Update, update_collect_properties);
        app.add_systems(Update, update_shaders);

        register_loading::<CharacterSpawn>(app);
        register_loading::<SkinAnimationSpawn>(app);
        register_loading::<SkinMeshSpawn>(app);
        register_loading::<SkinSkeletonSpawn>(app);
        register_loading::<SkinSpawn>(app);

        app.add_observer(on_command_load_prop_bin);

        let resource_cache = ResourceCache::default();

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
pub struct LeaguePropertyFiles {
    pub unload: Vec<(String, Handle<LeagueProperties>)>,
    pub loaded: Vec<String>,
}

#[derive(Resource, Default)]
pub struct ResourceCache {
    image: HashMap<String, Handle<Image>>,
    mesh: HashMap<String, Handle<Mesh>>,
}

#[derive(Event)]
pub struct CommandLoadPropBin {
    pub paths: Vec<String>,
}

#[derive(Component, Resource)]
pub struct Loading<F> {
    pub timer: Timer,
    pub value: F,
}

impl<F> Loading<F> {
    pub fn new(value: F) -> Self {
        Self {
            timer: Timer::from_seconds(10.0, TimerMode::Once),
            value,
        }
    }

    pub fn is_timeout(&self) -> bool {
        self.timer.is_finished()
    }

    pub fn update(&mut self, time: &Time) {
        self.timer.tick(time.delta());
    }

    pub fn set(&mut self, value: F) {
        self.value = value;
    }
}

impl<T> Deref for Loading<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
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

fn on_command_load_prop_bin(
    event: On<CommandLoadPropBin>,
    res_asset_server: Res<AssetServer>,
    mut res_league_property_files: ResMut<LeaguePropertyFiles>,
) {
    for path in &event.paths {
        let lower = path.to_lowercase();
        if res_league_property_files.loaded.contains(&lower) {
            continue;
        }

        res_league_property_files
            .unload
            .push((lower.clone(), res_asset_server.load_league(&lower)));

        res_league_property_files.loaded.push(lower);
    }
}

fn update_collect_properties(
    mut commands: Commands,
    mut res_assets_league_properties: ResMut<Assets<LeagueProperties>>,
    mut res_league_property_files: ResMut<LeaguePropertyFiles>,
    mut res_league_properties: ResMut<LeagueProperties>,
) {
    if res_league_property_files.unload.is_empty() {
        return;
    }

    res_league_property_files
        .unload
        .retain(|(_path, handle_league_properties)| {
            let Some(league_properties) =
                res_assets_league_properties.get_mut(handle_league_properties)
            else {
                return true;
            };

            res_league_properties.merge(league_properties);

            commands.trigger(CommandLoadPropBin {
                paths: league_properties.1.clone(),
            });

            false
        });

    if res_league_properties.0.is_empty() {
        return;
    }

    commands.queue(insert_props);
}

fn insert_props(world: &mut World) {
    let res_league_properties = world.resource::<LeagueProperties>();

    let collect = res_league_properties
        .0
        .iter()
        .flat_map(|(type_hash, v)| {
            v.iter()
                .map(|(prop_hash, v)| (type_hash.clone(), prop_hash.clone(), v.clone()))
        })
        .collect::<Vec<_>>();

    for (type_hash, prop_hash, mut handle) in collect {
        let (_, loader) = ASSET_LOADER_REGISTRY.loaders.get(&type_hash).unwrap();
        loader.load(world, prop_hash, &mut handle);
    }

    let mut res_league_properties = world.resource_mut::<LeagueProperties>();
    res_league_properties.0.clear();
}

pub trait LoadingTrait {
    fn is_timeout(&self) -> bool;
}

fn register_loading<T: TypePath + Send + Sync + 'static>(app: &mut App) {
    app.add_systems(Update, update_loading::<T>);
}

fn update_loading<T: TypePath + Send + Sync + 'static>(
    mut commands: Commands,
    mut q_loading: Query<(Entity, &mut Loading<T>)>,
    time: Res<Time>,
) {
    for (entity, mut loading) in q_loading.iter_mut() {
        loading.update(&time);

        if loading.is_timeout() {
            // println!("加载超时 {}", T::type_path());
            commands.entity(entity).remove::<Loading<T>>();
        }
    }
}
