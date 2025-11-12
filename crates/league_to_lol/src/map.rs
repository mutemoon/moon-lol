use std::collections::HashMap;

use bevy::prelude::*;

use league_core::{
    BarracksConfig, MapPlaceableContainer, MissileSpecificationBehaviors, Unk0x9d9f60d2,
};
use league_loader::{LeagueLoader, LeagueWadMapLoader};
use league_property::from_entry;
use league_utils::hash_bin;
use lol_config::{ConfigMap, ConfigNavigationGrid};
use lol_core::Lane;

use crate::{
    get_bin_path, load_navigation_grid, save_character, save_legends, save_mapgeo,
    save_struct_to_file, Error, CONFIG_PATH_MAP, CONFIG_PATH_MAP_NAV_GRID,
};

pub async fn save_config_map(
    root_dir: &str,
    map: &str,
    champions: Vec<(&str, &str)>,
) -> Result<ConfigMap, Error> {
    let loader = LeagueLoader::new(root_dir, map)?;
    let map_loader = loader.map_loader;
    // 并行处理地图网格
    let geometry_objects = save_mapgeo(&map_loader).await?;

    let mut minion_paths = HashMap::new();
    let mut barracks = HashMap::new();
    let mut characters = HashMap::new();
    let mut barrack_configs = HashMap::new();
    let mut environment_objects = HashMap::new();
    let mut vfx_system_definition_datas = HashMap::new();

    for entry in map_loader
        .materials_bin
        .iter_entry_by_class(hash_bin("BarracksConfig"))
    {
        barrack_configs
            .entry(entry.hash)
            .or_insert(from_entry::<BarracksConfig>(entry));
    }

    for entry in map_loader.materials_bin.iter_entry_by_class(0x9d9f60d2) {
        let record = from_entry::<Unk0x9d9f60d2>(entry);
        characters.entry(entry.hash).or_insert(record);
    }

    for entry in map_loader
        .materials_bin
        .iter_entry_by_class(hash_bin("MapPlaceableContainer"))
    {
        let map_placeable_container = from_entry::<MapPlaceableContainer>(entry);

        for (hash, value) in map_placeable_container.items {
            match value {
                MissileSpecificationBehaviors::Unk0xad65d8c4(unk0xad65d8c4) => {
                    environment_objects.entry(hash).or_insert(unk0xad65d8c4);
                }
                MissileSpecificationBehaviors::Unk0x3c995caf(unk0x3c995caf) => {
                    let lane = match unk0x3c995caf.name.as_str() {
                        "MinionPath_Top" => Lane::Top,
                        "MinionPath_Mid" => Lane::Mid,
                        "MinionPath_Bot" => Lane::Bot,
                        _ => panic!("未知的小兵路径: {}", unk0x3c995caf.name),
                    };

                    let translation = unk0x3c995caf.transform.to_scale_rotation_translation().2;

                    let path = unk0x3c995caf
                        .segments
                        .iter()
                        .map(|v| (v + translation).xz())
                        .collect();

                    minion_paths.entry(lane).or_insert(path);
                }
                MissileSpecificationBehaviors::Unk0xba138ae3(unk0xba138ae3) => {
                    barracks.entry(hash).or_insert(unk0xba138ae3);
                }
                _ => {}
            }
        }
    }

    let mut skin_and_record_pairs = Vec::new();

    for environment_object in environment_objects.values() {
        skin_and_record_pairs.push((
            environment_object.definition.skin.clone(),
            environment_object.definition.character_record.clone(),
        ));
    }

    for character in characters.values() {
        skin_and_record_pairs.push((
            character.skin.clone(),
            character.character_record.clone(),
        ));
    }

    for (skin_path, record_path) in skin_and_record_pairs {
        let skin_vfx_system_definition_datas =
            save_character(&map_loader.wad_loader, &skin_path, &record_path).await?;

        vfx_system_definition_datas.extend(skin_vfx_system_definition_datas);
    }

    for (champion, skin) in champions {
        let champion_vfx_system_definition_datas = save_legends(root_dir, champion, skin).await?;
        vfx_system_definition_datas.extend(champion_vfx_system_definition_datas);
    }

    save_navigation_grid(&map_loader).await?;

    let configs = ConfigMap {
        geometry_objects,
        minion_paths,
        barracks,
        characters,
        barrack_configs,
        environment_objects,
        vfx_system_definition_datas,
    };

    // 保存最终配置文件
    let path = get_bin_path(CONFIG_PATH_MAP);
    save_struct_to_file(&path, &configs).await?;

    Ok(configs)
}

pub async fn save_navigation_grid(
    loader: &LeagueWadMapLoader,
) -> Result<ConfigNavigationGrid, Error> {
    let nav_grid = load_navigation_grid(loader).await?;
    let path = get_bin_path(CONFIG_PATH_MAP_NAV_GRID);
    save_struct_to_file(&path, &nav_grid).await?;
    Ok(nav_grid)
}
