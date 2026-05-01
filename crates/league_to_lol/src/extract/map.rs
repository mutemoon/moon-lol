use std::collections::HashMap;

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use league_core::extract::{
    BarracksConfig, EnumMap, MapContainer, MapPlaceableContainer, StaticMaterialDef,
};
use league_file::grid::AiMeshNGrid;
use league_file::mapgeo::LeagueMapGeo;
use league_loader::game::{Data, LeagueLoader};
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_property::extract::get_hashes;
use lol_base::character::{ConfigCharacterRecord, ConfigSkin};
use lol_base::map::MapPaths;
use lol_core::entities::barrack::BarrackConfigHandler;
use lol_core::lane::Lane;
use lol_core::map::MinionPath;
use lol_core::navigation::grid::ResourceGrid;
use lol_core::team::Team;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::barrack::barracks_config_to_barracks;
use crate::extract::champion::{
    ChampionRecordData, extract_character_from_record, skin_path_to_skin_bin_path,
};
use crate::extract::utils::write_to_file;
use crate::gltf_export::export_mapgeo_to_gltf;
use crate::navgrid::load_league_nav_grid;

/// 完整的地图提取结果
pub struct MapExtractResult {
    pub minion_path: MinionPath,
}

/// Phase 1: 创建 Loader
pub fn extract_phase_1_create_loader(game_path: &str) -> LeagueLoader {
    println!("[1/7] Phase 1: 扫描 WAD 文件并创建 Loader...");

    let mut champion_wad_files = Vec::new();
    let champions_path = std::path::Path::new(game_path).join("DATA/FINAL/Champions");
    if let Ok(entries) = std::fs::read_dir(&champions_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.ends_with(".wad.client") && !name.contains('_') {
                    champion_wad_files.push(format!("DATA/FINAL/Champions/{}", name));
                }
            }
        }
    }

    let mut wad_files: Vec<&str> = vec![
        "DATA/FINAL/UI.wad.client",
        "DATA/FINAL/UI.zh_CN.wad.client",
        "DATA/FINAL/Maps/Shipping/Map11.wad.client",
        "DATA/FINAL/Bootstrap.windows.wad.client",
    ];
    wad_files.extend(champion_wad_files.iter().map(|s| s.as_str()));

    LeagueLoader::from_relative_path(game_path, wad_files)
}

/// Phase 2: 提取所有英雄
pub fn extract_phase_2_champions(loader: &LeagueLoader, hashes: &HashMap<u32, String>) {
    println!("[2/7] Phase 2: 提取所有英雄...");
    let champions_path = std::path::Path::new(&loader.root_dir).join("DATA/FINAL/Champions");
    let Ok(entries) = std::fs::read_dir(&champions_path) else {
        eprintln!(
            "[ERROR] 无法读取 Champions 目录: {:?}",
            champions_path.display()
        );
        return;
    };

    let wad_entries: Vec<_> = entries.flatten().collect();
    println!("[INFO] 发现 {} 个 WAD 文件", wad_entries.len());

    let character_names: Vec<String> = wad_entries
        .iter()
        .filter_map(|entry| {
            let path = entry.path();
            let file_name = path.file_name()?.to_str()?;
            if file_name.ends_with(".wad.client") && !file_name.contains('_') {
                Some(file_name.trim_end_matches(".wad.client").to_lowercase())
            } else {
                None
            }
        })
        .collect();

    println!("[INFO] 开始并行提取 {} 个英雄...", character_names.len());

    let results: Vec<(String, bool)> = character_names
        .into_par_iter()
        .map(|character_name: String| {
            let skin_bin_path = Some(format!(
                "data/characters/{}/skins/skin0.bin",
                character_name
            ));
            let success = extract_character_from_record(
                loader,
                &character_name,
                true,
                skin_bin_path.as_deref(),
                hashes,
            );
            (character_name, success)
        })
        .collect();

    let success_count = results.iter().filter(|(_, success)| *success).count();
    let skip_count = results.len() - success_count;

    // 只打印失败项
    for (character_name, success) in &results {
        if !*success {
            println!("[WARN] 跳过: {}", character_name);
        }
    }

    println!(
        "[SUMMARY] 英雄提取完成: 成功 {} 个, 跳过 {} 个",
        success_count, skip_count
    );
}

/// Phase 3: 提取地图块数据（兵线、兵营、角色记录）
pub fn extract_phase_3_map_chunks(
    world: &mut World,
    loader: &LeagueLoader,
    map_paths: &MapPaths,
) -> std::collections::HashMap<String, Vec<ChampionRecordData>> {
    println!("[3/7] Phase 3: 提取地图块数据...");
    let prop_group = loader
        .get_prop_group_by_paths(vec![
            &map_paths.materials_bin_path(),
            "data/maps/shipping/map11/map11.bin",
        ])
        .unwrap();

    let map_container = prop_group.get_data::<MapContainer>(map_paths.materials_path());

    let mut minion_path = MinionPath::default();
    let mut map_character_records: std::collections::HashMap<String, Vec<ChampionRecordData>> =
        std::collections::HashMap::new();

    for (_, &link) in &map_container.chunks {
        let map_placeable_container = prop_group.get_data::<MapPlaceableContainer>(link);

        let Some(items) = map_placeable_container.items.as_ref() else {
            continue;
        };

        for (_, value) in items {
            match value {
                EnumMap::Unk0x3c995caf(unk0x3c995caf) => {
                    let lane = match unk0x3c995caf.name.as_str() {
                        "MinionPath_Top" => Lane::Top,
                        "MinionPath_Mid" => Lane::Mid,
                        "MinionPath_Bot" => Lane::Bot,
                        "TopLaneHomeguardsPath" => Lane::Top,
                        "MidLaneHomeguardsPath" => Lane::Mid,
                        "BotLaneHomeguardsPath" => Lane::Bot,
                        _ => panic!("未知的小兵路径: {}", unk0x3c995caf.name),
                    };

                    let translation = unk0x3c995caf
                        .transform
                        .unwrap_or_default()
                        .to_scale_rotation_translation()
                        .2;

                    let path = unk0x3c995caf
                        .segments
                        .iter()
                        .map(|v| (v + translation).xz())
                        .collect();

                    minion_path.0.entry(lane).or_insert(path);
                }
                EnumMap::Unk0xba138ae3(unk0xba138ae3) => {
                    let barracks_config_hash = unk0xba138ae3.barracks.barracks_config;
                    let barracks_config =
                        prop_group.get_data::<BarracksConfig>(barracks_config_hash);

                    let config_barracks = barracks_config_to_barracks(barracks_config.clone());
                    let ron_content = ron::to_string(&config_barracks).unwrap();
                    write_to_file(
                        &map_paths.barracks_ron_export(barracks_config_hash),
                        ron_content,
                    );

                    world.spawn((
                        Transform::from_matrix(unk0xba138ae3.transform),
                        Team::from(unk0xba138ae3.team.as_ref().map(|x| x.team)),
                        Lane::from(unk0xba138ae3.barracks.lane),
                        BarrackConfigHandler::new(
                            world
                                .resource::<AssetServer>()
                                .load(&map_paths.barracks_ron(barracks_config_hash)),
                        ),
                    ));
                }
                EnumMap::Unk0xad65d8c4(unk0xad65d8c4) => {
                    let transform = Transform::from_matrix(unk0xad65d8c4.transform.unwrap());

                    // 从路径提取英雄名
                    let character_name = unk0xad65d8c4
                        .character
                        .character_record
                        .split('/')
                        .skip(1)
                        .next();

                    if let Some(character_name) = character_name {
                        // Extract skin_id from skin path (e.g., "Characters/Aatrox/Skins/Skin0" -> "skin0")
                        let skin_id = unk0xad65d8c4
                            .character
                            .skin
                            .split('/')
                            .last()
                            .unwrap_or("skin0")
                            .to_lowercase();

                        map_character_records
                            .entry(character_name.to_lowercase())
                            .or_insert_with(Vec::new)
                            .push(ChampionRecordData {
                                char_record_path: unk0xad65d8c4.character.character_record.clone(),
                                skin_path: Some(unk0xad65d8c4.character.skin.clone()),
                            });

                        world.spawn((
                            transform,
                            Team::from(unk0xad65d8c4.team.as_ref().map(|x| x.team)),
                            ConfigSkin {
                                skin: world.resource::<AssetServer>().load(&format!(
                                    "characters/{}/skins/{}.ron",
                                    character_name, skin_id
                                )),
                            },
                            ConfigCharacterRecord {
                                character_record: world
                                    .resource::<AssetServer>()
                                    .load(&format!("characters/{}/config.ron", character_name)),
                            },
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    world.insert_resource(minion_path);
    map_character_records
}

/// Phase 4: 提取导航网格
pub fn extract_phase_4_nav_grid(world: &mut World, loader: &LeagueLoader, map_paths: &MapPaths) {
    println!("[4/7] Phase 4: 提取导航网格...");
    let prop_group = loader
        .get_prop_group_by_paths(vec![
            &map_paths.materials_bin_path(),
            "data/maps/shipping/map11/map11.bin",
        ])
        .unwrap();

    let map_container = prop_group.get_data::<MapContainer>(map_paths.materials_path());

    if let Some(map_nav_grid) = map_container.components.iter().find_map(|item| {
        if let EnumMap::MapNavGrid(map_nav_grid) = item {
            Some(map_nav_grid)
        } else {
            None
        }
    }) {
        let nav_grid_path = map_nav_grid.nav_grid_path.to_string();
        let buf = loader.get_wad_entry_buffer_by_path(&nav_grid_path).unwrap();
        let (_, nav_grid) = AiMeshNGrid::parse(&buf).unwrap();
        let config_navigation_grid = load_league_nav_grid(nav_grid);

        crate::extract::utils::write_bin_to_file(
            &map_paths.navgrid_bin_export(),
            &config_navigation_grid,
        );

        world.insert_resource(ResourceGrid(
            world
                .resource::<AssetServer>()
                .load(&map_paths.navgrid_bin()),
        ));
    }
}

/// Phase 5: 导出地图几何到 GLTF
pub fn extract_phase_5_map_geo(loader: &LeagueLoader, map_paths: &MapPaths) {
    println!("[5/7] Phase 5: 导出地图几何到 GLTF...");
    let prop_group = loader
        .get_prop_group_by_paths(vec![
            &map_paths.materials_bin_path(),
            "data/maps/shipping/map11/map11.bin",
        ])
        .unwrap();

    let buf = loader
        .get_wad_entry_buffer_by_path(&map_paths.mapgeo_path())
        .unwrap();
    let (_, league_mapgeo) = LeagueMapGeo::parse(&buf).unwrap();

    let mut material_defs = std::collections::HashMap::new();
    for mesh in &league_mapgeo.meshes {
        for submesh in &mesh.submeshes {
            let mat_name = &submesh.material_name.text;
            let mat_hash = league_utils::hash_bin(mat_name);
            if let Some(mat_def) = prop_group.get_data_option::<StaticMaterialDef>(mat_hash) {
                material_defs.insert(mat_hash, mat_def);
            }
        }
    }

    export_mapgeo_to_gltf(
        &league_mapgeo,
        &map_paths.mapgeo_glb_export().replace(".glb", ""),
        &material_defs,
        loader,
    )
    .unwrap();
}

/// Phase 6: 从地图中提取角色记录
pub fn extract_phase_6_map_character_records(
    loader: &LeagueLoader,
    map_character_records: &std::collections::HashMap<String, Vec<ChampionRecordData>>,
    hashes: &HashMap<u32, String>,
) {
    if map_character_records.is_empty() {
        return;
    }

    println!(
        "[6/7] Phase 6: 从地图中提取 {} 个角色记录...",
        map_character_records.len()
    );

    let items: Vec<(String, Vec<ChampionRecordData>)> = map_character_records
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let results: Vec<(String, bool)> = items
        .into_par_iter()
        .map(|(character_name, records)| {
            let mut results = Vec::new();
            for record_data in records {
                let skin_bin_path = record_data
                    .skin_path
                    .as_ref()
                    .map(|skin_path| skin_path_to_skin_bin_path(&character_name, skin_path));
                let success = extract_character_from_record(
                    loader,
                    &character_name,
                    false,
                    skin_bin_path.as_deref(),
                    hashes,
                );
                results.push((character_name.clone(), success));
            }
            results
        })
        .flatten()
        .collect();

    let success_count = results.iter().filter(|(_, success)| *success).count();

    // 只打印失败项
    for (character_name, success) in &results {
        if !*success {
            println!("[WARN] 跳过: {}", character_name);
        }
    }

    println!("[SUMMARY] 地图角色记录提取完成: 成功 {} 个", success_count);
}

/// Phase 7: 序列化 World 到文件
pub fn extract_phase_7_serialize_world(world: &mut World, map_paths: &MapPaths) {
    println!("[7/7] Phase 7: 序列化 World 到文件...");
    let scene = DynamicWorld::from_world(world);
    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();
    let serialized_scene = scene.serialize(&type_registry).unwrap();

    write_to_file(&map_paths.scene_ron_export(), &serialized_scene);
}

/// 一键提取所有数据（英雄 + 地图）
pub fn extract_all(game_path: &str, hashes_dir: &str) {
    let loader = extract_phase_1_create_loader(game_path);
    let map_paths = MapPaths::default();

    // 加载 hash 对照表
    let hash_paths = vec![
        format!("{}/hashes.binentries.txt", hashes_dir),
        format!("{}/hashes.binfields.txt", hashes_dir),
        format!("{}/hashes.binhashes.txt", hashes_dir),
        format!("{}/hashes.bintypes.txt", hashes_dir),
    ];
    let hashes = get_hashes(&hash_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    let mut app = App::new();
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(TaskPoolPlugin::default());
    app.init_asset::<DynamicWorld>();
    app.init_asset::<lol_base::barrack::ConfigBarracks>();
    app.init_asset::<lol_base::grid::ConfigNavigationGrid>();
    app.init_asset_loader::<lol_core::loaders::barrack::ConfigBarracksLoader>();
    app.finish();
    app.cleanup();

    let world = app.world_mut();

    // Phase 2: 提取英雄
    extract_phase_2_champions(&loader, &hashes);

    // Phase 3: 提取地图块
    let map_character_records = extract_phase_3_map_chunks(world, &loader, &map_paths);

    // Phase 4: 提取导航网格
    extract_phase_4_nav_grid(world, &loader, &map_paths);

    // Phase 5: 导出地图几何
    extract_phase_5_map_geo(&loader, &map_paths);

    // Phase 6: 从地图提取角色记录
    extract_phase_6_map_character_records(&loader, &map_character_records, &hashes);

    // Phase 7: 序列化 World
    extract_phase_7_serialize_world(world, &map_paths);
}
