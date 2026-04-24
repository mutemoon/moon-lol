use bevy::asset::AssetPath;
use bevy::ecs::archetype;
use bevy::prelude::*;
use league_core::extract::{
    BarracksConfig, CharacterRecord, EnumMap, MapContainer, MapPlaceableContainer,
    SkinCharacterDataProperties, StaticMaterialDef,
};
use league_file::grid::AiMeshNGrid;
use league_file::mapgeo::LeagueMapGeo;
use league_file::mesh_skinned::LeagueSkinnedMesh;
use league_loader::game::{Data, LeagueLoader};
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_to_lol::barrack::barracks_config_to_barracks;
use league_to_lol::gltf_export::export_mapgeo_to_gltf;
use league_to_lol::navgrid::load_league_nav_grid;
use league_to_lol::skin_gltf_export::{decode_texture_to_png, export_skin_to_glb};
use lol_base::barrack::ConfigBarracks;
use lol_base::character::{ConfigCharacterRecord, ConfigSkin, HealthBar, Skin};
use lol_base::grid::ConfigNavigationGrid;
use lol_core::attack::{Attack, WindupConfig};
use lol_core::base::bounding::Bounding;
use lol_core::base::level::ExperienceDrop;
use lol_core::character::Character;
use lol_core::damage::{Armor, Damage};
use lol_core::entities::barrack::BarrackConfigHandler;
use lol_core::entities::champion::Champion;
use lol_core::lane::Lane;
use lol_core::life::Health;
use lol_core::map::{MapName, MinionPath};
use lol_core::movement::Movement;
use lol_core::navigation::grid::ResourceGrid;
use lol_core::team::Team;
use lol_loader::barrack::BarracksLoader;
use serde::Serialize;

#[derive(Clone)]
struct ChampionRecordData {
    char_record_path: String,
    skin_path: Option<String>,
}

/// 将 skin 路径转换为 skin bin 文件路径
/// 例如: "Characters/Aatrox/Skins/Skin0" -> "data/characters/aatrox/skins/skin0.bin"
fn skin_path_to_skin_bin_path(champ_name: &str, skin_path: &str) -> String {
    // skin_path 格式: "Characters/{Name}/Skins/{SkinName}"
    // 例如: "Characters/Aatrox/Skins/Skin0"
    // 输出: "data/characters/{name}/skins/{skinname}.bin"
    let parts: Vec<&str> = skin_path.split('/').collect();
    if parts.len() >= 4 {
        let skin_name = parts[parts.len() - 1].to_lowercase();
        format!(
            "data/characters/{}/skins/{}.bin",
            champ_name.to_lowercase(),
            skin_name
        )
    } else {
        // Fallback: 假设格式是 "{SkinName}"，直接拼接
        format!(
            "data/characters/{}/skins/{}.bin",
            champ_name.to_lowercase(),
            skin_path.to_lowercase()
        )
    }
}

fn main() {
    let mut app = App::new();

    app.add_plugins(AssetPlugin::default());
    app.add_plugins(TaskPoolPlugin::default());

    app.init_asset::<DynamicWorld>();
    app.init_asset::<ConfigBarracks>();
    app.init_asset::<ConfigNavigationGrid>();
    app.init_asset_loader::<BarracksLoader>();

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
    let mut map_character_records: std::collections::HashMap<String, ChampionRecordData> =
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
                        &format!("assets/maps/{}.ron", barracks_config_hash),
                        ron_content,
                    );

                    world.spawn((
                        Transform::from_matrix(unk0xba138ae3.transform),
                        Team::from(unk0xba138ae3.team.as_ref().map(|x| x.team)),
                        Lane::from(unk0xba138ae3.barracks.lane),
                        BarrackConfigHandler::new(
                            world
                                .resource::<AssetServer>()
                                .load(&format!("maps/{}.ron", barracks_config_hash)),
                        ),
                    ));
                }
                EnumMap::Unk0xad65d8c4(unk0xad65d8c4) => {
                    let transform = Transform::from_matrix(unk0xad65d8c4.transform.unwrap());

                    // 从路径提取英雄名，如 "Characters/Aatrox/CharacterRecords/Root" -> "Aatrox"
                    let champ_name = unk0xad65d8c4
                        .character
                        .character_record
                        .split('/')
                        .skip(1)
                        .next();

                    if let Some(champ_name) = champ_name {
                        map_character_records.insert(
                            champ_name.to_lowercase(),
                            ChampionRecordData {
                                char_record_path: unk0xad65d8c4.character.character_record.clone(),
                                skin_path: Some(unk0xad65d8c4.character.skin.clone()),
                            },
                        );

                        world.spawn((
                            transform,
                            Team::from(unk0xad65d8c4.team.as_ref().map(|x| x.team)),
                            ConfigSkin {
                                skin: world
                                    .resource::<AssetServer>()
                                    .load(&format!("characters/{}/skin.ron", champ_name)),
                            },
                            ConfigCharacterRecord {
                                character_record: world
                                    .resource::<AssetServer>()
                                    .load(&format!("characters/{}/config.ron", champ_name)),
                            },
                        ));
                    }
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

            world.insert_resource(ResourceGrid(
                world
                    .resource::<AssetServer>()
                    .load(&format!("maps/{}_navgrid.bin", map_name)),
            ));
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

    let scene = DynamicWorld::from_world(world);
    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();
    let serialized_scene = scene.serialize(&type_registry).unwrap();

    write_to_file(
        &format!("assets/{}", map_name.get_ron_path()),
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

fn write_bin_to_file<T: Serialize>(path: &str, content: &T) {
    write_to_file(
        path,
        bincode::serialize(content).expect("无法序列化为二进制"),
    );
}

/// 从 CharacterRecord 创建所有组件（占位符，待摸清对应关系）
fn create_champion_components_from_record(
    record: &CharacterRecord,
) -> (
    Attack,
    Health,
    Damage,
    Armor,
    Movement,
    Option<ExperienceDrop>,
) {
    // Attack
    let range = record.acquisition_range.unwrap_or(0.0);
    let windup_config = if let Some(basic_attack) = &record.basic_attack {
        let cast_time = basic_attack.m_attack_cast_time.unwrap_or(0.3);
        let total_time = basic_attack.m_attack_total_time.unwrap_or(0.7);
        WindupConfig::Modern {
            attack_cast_time: cast_time,
            attack_total_time: total_time,
        }
    } else {
        WindupConfig::Legacy { attack_offset: 0.3 }
    };
    let attack = match windup_config {
        WindupConfig::Modern {
            attack_cast_time,
            attack_total_time,
        } => Attack::new(range, attack_cast_time, attack_total_time),
        WindupConfig::Legacy { attack_offset } => Attack::from_legacy(range, 1.0, attack_offset),
    };

    // TODO: 摸清字段对应关系后替换为正确的值
    // 占位符：使用 exp_given_on_death 作为 health（待替换）
    let health = Health::new(record.exp_given_on_death.unwrap_or(1000.0));

    // 占位符：使用 unk_0x43135375 作为 damage（待替换）
    let damage = Damage(record.unk_0x43135375.unwrap_or(50.0));

    // 占位符：使用 unk_0x4af40dc3 中的某个值作为 armor（待替换）
    let armor = Armor(record.area_indicator_radius.unwrap_or(30.0));

    // 占位符：使用 perception_bubble_radius 作为 move speed（待替换）
    let movement = Movement {
        speed: record.perception_bubble_radius.unwrap_or(5.0),
    };

    // 经验掉落
    let experience_drop =
        if let (Some(exp), Some(radius)) = (record.exp_given_on_death, record.experience_radius) {
            Some(ExperienceDrop {
                exp_given_on_death: exp,
                experience_radius: radius,
            })
        } else {
            None
        };

    (attack, health, damage, armor, movement, experience_drop)
}

/// 导出 champion 实体为 Scene 文件
fn export_champion_scene(
    champ_name: &str,
    bounding: Bounding,
    attack: Attack,
    health: Health,
    damage: Damage,
    armor: Armor,
    movement: Movement,
    experience_drop: Option<ExperienceDrop>,
) {
    let mut world = World::new();

    // 注册类型
    let type_registry = AppTypeRegistry::default();
    type_registry.write().register::<Champion>();
    type_registry.write().register::<Bounding>();
    type_registry.write().register::<Transform>();
    type_registry.write().register::<GlobalTransform>();
    type_registry.write().register::<Name>();
    type_registry.write().register::<Attack>();
    type_registry.write().register::<WindupConfig>();
    type_registry.write().register::<Health>();
    type_registry.write().register::<Damage>();
    type_registry.write().register::<Armor>();
    type_registry.write().register::<Movement>();
    type_registry.write().register::<ExperienceDrop>();
    world.insert_resource(type_registry);

    let champ_name_string = champ_name.to_string();
    let entity = world
        .spawn((
            Character,
            Name::new(champ_name_string),
            Champion,
            bounding,
            attack,
            health,
            damage,
            armor,
            movement,
        ))
        .id();

    if let Some(exp_drop) = experience_drop {
        world.entity_mut(entity).insert(exp_drop);
    }

    let scene = DynamicWorld::from_world(&world);
    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();
    let serialized_scene = scene.serialize(&type_registry).unwrap();

    let output_path = format!("assets/characters/{}/config.ron", champ_name.to_lowercase());
    write_to_file(&output_path, serialized_scene);
}

/// 从地图中收集的 CharacterRecord 路径提取数据
fn extract_character_records_from_map(
    loader: &LeagueLoader,
    map_character_records: &std::collections::HashMap<String, ChampionRecordData>,
) {
    if map_character_records.is_empty() {
        return;
    }

    println!(
        "[INFO] 从地图中提取 {} 个 CharacterRecord",
        map_character_records.len()
    );

    let mut success_count = 0;

    for (champ_name, record_data) in map_character_records {
        let skin_bin_path = record_data
            .skin_path
            .as_ref()
            .map(|skin_path| skin_path_to_skin_bin_path(champ_name, skin_path));
        if extract_champion_from_record(loader, champ_name, skin_bin_path.as_deref()) {
            println!("[OK] 提取成功: {}", champ_name);
            success_count += 1;
        }
    }

    println!(
        "[INFO] 地图 CharacterRecord 提取完成: 成功 {} 个",
        success_count
    );
}

/// 从 CharacterRecord 提取 champion 并导出场景
fn extract_champion_from_record(
    loader: &LeagueLoader,
    champ_name: &str,
    skin_bin_path: Option<&str>,
) -> bool {
    let bin_path = format!("data/characters/{}/{}.bin", champ_name, champ_name);

    let Ok(prop_group) = loader.get_prop_group_by_paths(vec![&bin_path]) else {
        println!("[WARN] 无法加载 bin 文件: {}", bin_path);
        return false;
    };

    let Some(record) = prop_group.get_by_class::<CharacterRecord>() else {
        println!("[WARN] 无法获取 CharacterRecord: {}", bin_path);
        return false;
    };

    let bounding = Bounding {
        radius: record.pathfinding_collision_radius.unwrap_or(0.0),
        height: record.health_bar_height.unwrap_or(200.0),
    };

    let (attack, health, damage, armor, movement, experience_drop) =
        create_champion_components_from_record(&record);
    export_champion_scene(
        champ_name,
        bounding,
        attack,
        health,
        damage,
        armor,
        movement,
        experience_drop,
    );

    // 提取皮肤 GLB 和皮肤场景
    extract_skin_for_champion(loader, champ_name, skin_bin_path);

    true
}

/// 导出角色的皮肤 GLB 和皮肤场景文件
fn extract_skin_for_champion(loader: &LeagueLoader, champ_name: &str, skin_bin_path: Option<&str>) {
    let Some(skin_bin_path) = skin_bin_path else {
        return;
    };

    // 加载皮肤 bin 文件获取 SkinCharacterDataProperties
    let Ok(skin_prop_group) = loader.get_prop_group_by_paths(vec![skin_bin_path]) else {
        println!("[WARN] 无法加载皮肤 bin 文件: {}", skin_bin_path);
        return;
    };

    let Some(skin_data) = skin_prop_group.get_by_class::<SkinCharacterDataProperties>() else {
        println!(
            "[WARN] 无法获取 SkinCharacterDataProperties: {}",
            skin_bin_path
        );
        return;
    };

    let skin_mesh_properties = match &skin_data.skin_mesh_properties {
        Some(props) => props,
        None => return,
    };

    let simple_skin_path = match &skin_mesh_properties.simple_skin {
        Some(path) => path,
        None => return,
    };

    let texture_path = match &skin_mesh_properties.texture {
        Some(path) => path.clone(),
        None => return,
    };

    // 加载 .skn 文件
    let skn_buf = match loader.get_wad_entry_buffer_by_path(simple_skin_path) {
        Ok(buf) => buf,
        Err(_) => {
            println!("[WARN] 无法加载 SKN 文件: {}", simple_skin_path);
            return;
        }
    };

    let (_, skinned_mesh) = match LeagueSkinnedMesh::parse(&skn_buf) {
        Ok(mesh) => mesh,
        Err(_) => {
            println!("[WARN] 无法解析 SKN 文件: {}", simple_skin_path);
            return;
        }
    };

    // 加载 .tex 文件并解码为 PNG
    let texture_png = loader
        .get_wad_entry_buffer_by_path(&texture_path)
        .ok()
        .and_then(|buf| {
            let (_, texture) = league_file::texture::LeagueTexture::parse(&buf).ok()?;
            decode_texture_to_png(&texture)
        });

    let output_glb_path = format!("assets/characters/{}/skin.glb", champ_name.to_lowercase());
    if let Err(e) = export_skin_to_glb(&skinned_mesh, texture_png, &output_glb_path) {
        println!("[WARN] 皮肤 GLB 导出失败: {}", e);
        return;
    }

    // 获取 scale 和 bar_type
    let scale = skin_mesh_properties.skin_scale.unwrap_or(1.0);
    let bar_type = skin_data
        .health_bar_data
        .as_ref()
        .and_then(|h| h.unit_health_bar_style)
        .unwrap_or(0);

    // 构建皮肤场景 skin.ron
    let mut app = App::new();

    app.add_plugins(AssetPlugin::default());
    app.add_plugins(TaskPoolPlugin::default());

    app.init_asset::<WorldAsset>();

    app.finish();
    app.cleanup();

    let world = app.world_mut();

    let asset_server = world.resource::<AssetServer>();
    let skin_handle: Handle<WorldAsset> = asset_server.load(
        AssetPath::from(format!("characters/{}/skin.glb", champ_name.to_lowercase()))
            .with_label(GltfAssetLabel::Scene(0).to_string()),
    );

    let _entity = world
        .spawn((
            Skin { scale },
            HealthBar { bar_type },
            Visibility::default(),
            WorldAssetRoot(skin_handle),
        ))
        .id();

    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();
    let scene = DynamicWorldBuilder::from_world(&world, &type_registry)
        .deny_component::<InheritedVisibility>()
        .deny_component::<ViewVisibility>()
        .deny_component::<GlobalTransform>()
        .deny_component::<Transform>()
        .deny_component::<TransformTreeChanged>()
        .extract_entities(
            // we do this instead of a query, in order to completely sidestep default query filters.
            // while we could use `Allow<_>`, this wouldn't account for custom disabled components
            world
                .archetypes()
                .iter()
                .flat_map(archetype::Archetype::entities)
                .map(archetype::ArchetypeEntity::id),
        )
        .extract_resources()
        .build();
    let serialized_scene = scene.serialize(&type_registry).unwrap();

    let output_skin_path = format!("assets/characters/{}/skin.ron", champ_name.to_lowercase());
    write_to_file(&output_skin_path, serialized_scene);

    println!("[OK] 皮肤导出成功: {}", champ_name);
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
        let champ_name = file_name.trim_end_matches(".wad.client").to_lowercase();

        // champion 类型角色默认导出 Skin0
        let skin_bin_path = Some(format!("data/characters/{}/skins/skin0.bin", champ_name));
        if extract_champion_from_record(loader, &champ_name, skin_bin_path.as_deref()) {
            println!("[OK] 提取成功: {}", champ_name);
            success_count += 1;
        } else {
            skip_count += 1;
        }
    }

    println!(
        "[SUMMARY] 提取完成: 成功 {} 个, 跳过 {} 个",
        success_count, skip_count
    );
}
