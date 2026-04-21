use bevy::prelude::*;
use league_core::extract::{
    BarracksConfig, EnumMap, MapContainer, MapPlaceableContainer, StaticMaterialDef, Unk0xad65d8c4,
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

    let loader = LeagueLoader::from_relative_path(
        r"D:\WeGameApps\英雄联盟\Game",
        vec![
            "DATA/FINAL/UI.wad.client",
            "DATA/FINAL/UI.zh_CN.wad.client",
            "DATA/FINAL/Maps/Shipping/Map11.wad.client",
            "DATA/FINAL/Champions/Riven.wad.client",
            "DATA/FINAL/Champions/Fiora.wad.client",
            "DATA/FINAL/Bootstrap.windows.wad.client",
        ],
    );

    let map_name = MapName::default();

    let prop_group = loader
        .get_prop_group_by_paths(vec![
            &map_name.get_materials_bin_path(),
            "data/maps/shipping/map11/map11.bin",
        ])
        .unwrap();

    let map_container = prop_group.get_data::<MapContainer>(map_name.get_materials_path());

    let mut minion_path = MinionPath::default();

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

                    for unit in &barracks_config.units {
                        let character = prop_group.get_data::<Unk0xad65d8c4>(unit.unk_0xfee040bc);

                        // commands.trigger(CommandCharacterLoad {
                        //     character_record: character.character.character_record.clone(),
                        // });
                    }

                    world.spawn((
                        Transform::from_matrix(unk0xba138ae3.transform),
                        Team::from(&unk0xba138ae3.team),
                        Lane::from(unk0xba138ae3.barracks.lane),
                        ConfigBarracks::from(barracks_config),
                    ));
                }
                EnumMap::Unk0xad65d8c4(unk0xad65d8c4) => {
                    let transform = Transform::from_matrix(unk0xad65d8c4.transform.unwrap());

                    world.spawn((
                        transform,
                        Team::from(&unk0xad65d8c4.team),
                        ConfigCharacter {
                            character_record: unk0xad65d8c4.character.character_record.clone(),
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
