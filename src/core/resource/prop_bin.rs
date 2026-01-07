use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use lol_config::{LeagueProperties, ASSET_LOADER_REGISTRY};

use crate::{AssetServerLoadLeague, HashPath};

pub struct PluginResourcePropBin;

impl Plugin for PluginResourcePropBin {
    fn build(&self, app: &mut App) {
        app.init_asset::<LeagueProperties>();

        app.init_resource::<LeagueProperties>();
        app.init_resource::<LeaguePropertyFiles>();
        app.init_resource::<ActivePropLoads>();

        app.add_systems(Update, update_collect_properties);

        app.add_observer(on_command_load_prop_bin);
    }
}

#[derive(Resource, Default)]
pub struct LeaguePropertyFiles {
    pub unload: Vec<Handle<LeagueProperties>>,
    pub loaded: HashSet<u64>,
}

#[derive(Resource, Default)]
pub struct ActivePropLoads {
    pub map: HashMap<String, Vec<Handle<LeagueProperties>>>,
}

#[derive(Event)]
pub struct EventLoadPropEnd {
    pub label: String,
}

#[derive(Debug, Clone)]
pub enum PropPath {
    Path(Vec<String>),
    Hash(Vec<u64>),
}

#[derive(Event, Debug, Clone)]
pub struct CommandLoadPropBin {
    pub path: PropPath,
    pub label: Option<String>,
}

impl PropPath {
    pub fn len(&self) -> usize {
        match self {
            PropPath::Path(paths) => paths.len(),
            PropPath::Hash(hashes) => hashes.len(),
        }
    }
}

fn on_command_load_prop_bin(
    event: On<CommandLoadPropBin>,
    res_asset_server: Res<AssetServer>,
    mut res_league_property_files: ResMut<LeaguePropertyFiles>,
    mut res_active_prop_loads: ResMut<ActivePropLoads>,
) {
    if let Some(label) = &event.label {
        info!("{} 配置文件开始加载，一共 {} 个", label, event.path.len());
    }

    let mut handles = Vec::new();
    let mut load = |hash_path: HashPath| {
        if res_league_property_files.loaded.contains(&hash_path.hash) {
            return;
        }

        let handle = res_asset_server.load_league(hash_path.clone());
        res_league_property_files.unload.push(handle.clone());
        handles.push(handle);

        res_league_property_files.loaded.insert(hash_path.hash);
    };

    match &event.path {
        PropPath::Path(paths) => {
            for path in paths {
                load(HashPath::from(path.to_lowercase().as_str()));
            }
        }
        PropPath::Hash(hashes) => {
            for &hash in hashes {
                load(HashPath::from(hash));
            }
        }
    }

    if !handles.is_empty() {
        if let Some(label) = &event.label {
            res_active_prop_loads
                .map
                .entry(label.clone())
                .or_default()
                .extend(handles);
        }
    } else {
        // 如果没有新资源加载，可能已经加载完成，或者本来就是空的
        // 这里我们不直接触发 EventLoadPropEnd，因为可能还有正在加载的资源
    }
}

fn update_collect_properties(
    mut commands: Commands,
    mut res_assets_league_properties: ResMut<Assets<LeagueProperties>>,
    mut res_league_property_files: ResMut<LeaguePropertyFiles>,
    mut res_league_properties: ResMut<LeagueProperties>,
    mut res_active_prop_loads: ResMut<ActivePropLoads>,
) {
    if res_league_property_files.unload.is_empty() {
        return;
    }

    res_league_property_files
        .unload
        .retain(|handle_league_properties| {
            let Some(league_properties) =
                res_assets_league_properties.get_mut(handle_league_properties)
            else {
                return true;
            };

            res_league_properties.merge(league_properties);

            if league_properties.1.is_empty() {
                return false;
            }

            // 递归加载时，目前无法得知原始 label，这可能是一个问题
            // 如果递归加载的资源也需要通知完成，我们需要在 ActivePropLoads 中跟踪
            commands.trigger(CommandLoadPropBin {
                path: PropPath::Path(league_properties.1.clone()),
                label: None,
            });

            false
        });

    // 处理带标签的加载任务
    let mut finished_labels = Vec::new();
    for (label, handles) in res_active_prop_loads.map.iter_mut() {
        handles.retain(|handle| !res_assets_league_properties.contains(handle));
        if handles.is_empty() {
            finished_labels.push(label.clone());
        }
    }

    for label in finished_labels {
        res_active_prop_loads.map.remove(&label);
        if !label.is_empty() {
            info!("{} 配置文件加载完成", label);
            commands.trigger(EventLoadPropEnd { label });
        }
    }

    if res_league_properties.0.is_empty() {
        return;
    }

    commands.queue(insert_props);
}

fn insert_props(world: &mut World) {
    let res_league_properties = world.remove_resource::<LeagueProperties>().unwrap();

    for (type_hash, store) in res_league_properties.0 {
        for (prop_hash, handle) in store {
            let (_, loader) = ASSET_LOADER_REGISTRY.loaders.get(&type_hash).unwrap();
            loader.load(world, prop_hash, &handle);
        }
    }

    world.init_resource::<LeagueProperties>();
}
