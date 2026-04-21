use bevy::prelude::*;
use league_core::extract::{
    BarracksConfig, CharacterRecord, EnumMap, MapContainer, MapPlaceableContainer,
    StaticMaterialDef,
};
use league_file::grid::AiMeshNGrid;
use league_file::mapgeo::LeagueMapGeo;
use league_loader::game::{Data, LeagueLoader};
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_to_lol::gltf_export::export_mapgeo_to_gltf;
use league_to_lol::navgrid::load_league_nav_grid;
use lol_base::barrack::ConfigBarracks;
use lol_base::character::ConfigCharacter;
use lol_base::grid::ConfigNavigationGrid;
use lol_core::base::bounding::Bounding;
use lol_core::lane::Lane;
use lol_core::map::{MapName, MinionPath};
use lol_core::navigation::grid::ResourceGridPath;
use lol_core::team::Team;
use ron::ser::PrettyConfig;
use serde::Serialize;

fn main() {
    let mut app = App::new();

    app.add_plugins(AssetPlugin::default());
    app.add_plugins(TaskPoolPlugin::default());

    app.init_asset::<ConfigNavigationGrid>();

    app.finish();

    app.cleanup();

    let world = app.world_mut();

    let mut champion_wad_files = Vec::new();
    let champions_path = std::path::Path::new(r"D:\WeGameApps\英雄联盟\Game\DATA\FINAL\Champions");
    if let Ok(entries) = std::fs::read_dir(champions_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            // 检查文件名是否以 .wad.client 结尾且不包含下划线（排除语言版本）
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

    let loader = LeagueLoader::from_relative_path(r"D:\WeGameApps\英雄联盟\Game", wad_files);

    // 提取所有英雄角色数据
    extract_all_champions(&loader);

    let map_name = MapName::default();

    let prop_group = loader
        .get_prop_group_by_paths(vec![
            &map_name.get_materials_bin_path(),
            "data/maps/shipping/map11/map11.bin",
        ])
        .unwrap();

    let map_container = prop_group.get_data::<MapContainer>(map_name.get_materials_path());

    let mut minion_path = MinionPath::default();
    let mut map_character_records: std::collections::HashMap<String, String> =
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
                    let barracks_config = prop_group
                        .get_data::<BarracksConfig>(unk0xba138ae3.barracks.barracks_config);

                    world.spawn((
                        Transform::from_matrix(unk0xba138ae3.transform),
                        Team::from(unk0xba138ae3.team.as_ref().map(|x| x.team)),
                        Lane::from(unk0xba138ae3.barracks.lane),
                        ConfigBarracks::from(barracks_config),
                    ));
                }
                EnumMap::Unk0xad65d8c4(unk0xad65d8c4) => {
                    let transform = Transform::from_matrix(unk0xad65d8c4.transform.unwrap());

                    // 收集 character_record 路径
                    let char_record_path = unk0xad65d8c4.character.character_record.clone();
                    // 从路径提取英雄名，如 "Characters/Aatrox/CharacterRecords/Root" -> "Aatrox"
                    if let Some(champ_name) = char_record_path
                        .strip_prefix("Characters/")
                        .and_then(|s| s.split('/').next())
                    {
                        map_character_records
                            .insert(champ_name.to_lowercase(), char_record_path.clone());
                    }

                    world.spawn((
                        transform,
                        Team::from(unk0xad65d8c4.team.as_ref().map(|x| x.team)),
                        ConfigCharacter {
                            character_record: char_record_path,
                            skin_path: unk0xad65d8c4.character.skin.clone(),
                        },
                    ));
                }
                _ => {}
            }
        }
    }

    world.insert_resource(minion_path);

    map_container
        .components
        .iter()
        .find_map(|item| {
            if let EnumMap::MapNavGrid(map_nav_grid) = item {
                Some(map_nav_grid)
            } else {
                None
            }
        })
        .map(|map_nav_grid| {
            let nav_grid_path = map_nav_grid.nav_grid_path.to_string();

            let buf = loader.get_wad_entry_buffer_by_path(&nav_grid_path).unwrap();

            let (_, nav_grid) = AiMeshNGrid::parse(&buf).unwrap();

            let config_navigation_grid = load_league_nav_grid(nav_grid);

            write_bin_to_file(
                &format!("assets/maps/{}_navgrid.bin", map_name),
                &config_navigation_grid,
            );

            world.insert_resource(ResourceGridPath(nav_grid_path));
        });

    let buf = loader
        .get_wad_entry_buffer_by_path(&map_name.get_mapgeo_path())
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
        &format!("assets/maps/{}_mapgeo", map_name),
        &material_defs,
        &loader,
    )
    .unwrap();

    // 从地图中提取 CharacterRecord（来自 Unk0xad65d8c4）
    extract_character_records_from_map(&loader, &map_character_records);

    let scene = DynamicWorld::from_world(&world);
    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();
    let serialized_scene = scene.serialize(&type_registry).unwrap();

    write_to_file(
        format!("assets/maps/{}_scene.ron", map_name).as_str(),
        &serialized_scene,
    );
}

fn write_to_file(path: &str, content: impl AsRef<[u8]>) {
    let path = std::path::Path::new(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("无法创建目录");
    }
    std::fs::write(path, content).expect("无法写入文件");
}

fn write_ron_to_file<T: Serialize>(path: &str, content: &T) {
    write_to_file(
        path,
        &ron::ser::to_string_pretty(content, PrettyConfig::new().compact_arrays(true))
            .expect("无法序列化为ron"),
    );
}

fn write_bin_to_file<T: Serialize>(path: &str, content: &T) {
    write_to_file(
        path,
        bincode::serialize(content).expect("无法序列化为二进制"),
    );
}

/// 从地图中收集的 CharacterRecord 路径提取数据
fn extract_character_records_from_map(
    loader: &LeagueLoader,
    map_character_records: &std::collections::HashMap<String, String>,
) {
    if map_character_records.is_empty() {
        return;
    }

    println!(
        "[INFO] 从地图中提取 {} 个 CharacterRecord",
        map_character_records.len()
    );

    let mut success_count = 0;

    for (champ_name_lower, char_record_path) in map_character_records {
        // 构造 bin 路径
        let bin_path = format!(
            "data/characters/{}/{}.bin",
            champ_name_lower, champ_name_lower
        );

        let Ok(prop_group) = loader.get_prop_group_by_paths(vec![&bin_path]) else {
            println!("[WARN] 无法加载 bin 文件: {}", bin_path);
            continue;
        };

        let Some(record) = prop_group.get_by_class::<CharacterRecord>() else {
            println!("[WARN] 无法获取 CharacterRecord: {}", char_record_path);
            continue;
        };

        let config = ChampionConfig {
            bounding: Bounding {
                radius: record.pathfinding_collision_radius.unwrap_or(0.0),
                height: record.health_bar_height.unwrap_or(200.0),
            },
            character_name: record.m_character_name.clone(),
            spells: record.spells.clone(),
            passive_spell: record.m_character_passive_spell,
            acquisition_range: record.acquisition_range,
            selection_radius: record.selection_radius,
            selection_height: record.selection_height,
        };

        let output_path = format!("assets/champions/{}/config.ron", champ_name_lower);
        write_ron_to_file(&output_path, &config);
        println!("[OK] 提取成功: {} -> {}", champ_name_lower, output_path);
        success_count += 1;
    }

    println!(
        "[INFO] 地图 CharacterRecord 提取完成: 成功 {} 个",
        success_count
    );
}

fn extract_all_champions(loader: &LeagueLoader) {
    let champions_path = std::path::Path::new(r"D:\WeGameApps\英雄联盟\Game\DATA\FINAL\Champions");
    let Ok(entries) = std::fs::read_dir(champions_path) else {
        eprintln!("[ERROR] 无法读取 Champions 目录: {:?}", champions_path);
        return;
    };

    let wad_entries: Vec<_> = entries.flatten().collect();
    println!("[INFO] 发现 {} 个 WAD 文件", wad_entries.len());

    let mut success_count = 0;
    let mut skip_count = 0;

    for entry in wad_entries {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };

        // 跳过非英文版 wad.client 文件
        if !file_name.ends_with(".wad.client") || file_name.contains('_') {
            continue;
        }

        // 从 "Aatrox.wad.client" 提取 "Aatrox"
        let champ_name = file_name.trim_end_matches(".wad.client");
        let champ_name_lower = champ_name.to_lowercase();

        let bin_path = format!(
            "data/characters/{}/{}.bin",
            champ_name_lower, champ_name_lower
        );

        let Ok(prop_group) = loader.get_prop_group_by_paths(vec![&bin_path]) else {
            println!("[WARN] 无法加载 bin 文件: {}", bin_path);
            skip_count += 1;
            continue;
        };

        // 通过 class hash 获取 CharacterRecord
        let Some(record) = prop_group.get_by_class::<CharacterRecord>() else {
            println!("[WARN] 无法获取 CharacterRecord: {}", bin_path);
            skip_count += 1;
            continue;
        };

        let config = ChampionConfig {
            bounding: Bounding {
                radius: record.pathfinding_collision_radius.unwrap_or(0.0),
                height: record.health_bar_height.unwrap_or(200.0),
            },
            character_name: record.m_character_name.clone(),
            spells: record.spells.clone(),
            passive_spell: record.m_character_passive_spell,
            acquisition_range: record.acquisition_range,
            selection_radius: record.selection_radius,
            selection_height: record.selection_height,
        };

        let output_path = format!("assets/champions/{}/config.ron", champ_name_lower);
        write_ron_to_file(&output_path, &config);
        println!("[OK] 提取成功: {} -> {}", champ_name, output_path);
        success_count += 1;
    }

    println!(
        "[SUMMARY] 提取完成: 成功 {} 个, 跳过 {} 个",
        success_count, skip_count
    );
}

#[derive(Serialize)]
struct ChampionConfig {
    bounding: Bounding,
    character_name: String,
    spells: Option<Vec<u32>>,
    passive_spell: Option<u32>,
    acquisition_range: Option<f32>,
    selection_radius: Option<f32>,
    selection_height: Option<f32>,
}
