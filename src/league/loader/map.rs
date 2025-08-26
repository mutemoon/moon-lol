use std::{
    collections::HashMap,
    io::{Cursor, Read},
};

use bevy::math::{vec2, Vec3Swizzles};
use binrw::{io::NoSeek, BinRead, BinWrite};
use tokio::io::AsyncWriteExt;

use crate::{
    core::{
        ConfigGeometryObject, ConfigMap, ConfigNavigationGrid, ConfigNavigationGridCell, Lane,
        CONFIG_PATH_MAP, CONFIG_PATH_MAP_NAV_GRID,
    },
    league::{
        from_entry, get_asset_writer, get_bin_path, parse_vertex_data, save_struct_to_file,
        submesh_to_intermediate, AiMeshNGrid, BarracksConfig, LayerTransitionBehavior,
        LeagueLoader, LeagueLoaderError, LeagueMapGeo, LeagueMaterial, LeagueWadLoader,
        MapContainer, MapContainerComponents, MapPlaceableContainer, MapPlaceableContainerItems,
        PropFile, StaticMaterialDef, Unk0x9d9f60d2,
    },
};

pub struct LeagueWadMapLoader {
    pub wad_loader: LeagueWadLoader,
    pub map_geo: LeagueMapGeo,
    pub materials_bin: PropFile,
}

impl LeagueWadMapLoader {
    pub fn from_loader(
        wad_loader: LeagueWadLoader,
        map: &str,
    ) -> Result<LeagueWadMapLoader, LeagueLoaderError> {
        let map_geo_path = format!("data/maps/mapgeometry/map11/{}.mapgeo", map);

        let entry = wad_loader
            .wad
            .get_entry(LeagueLoader::hash_wad(&map_geo_path))?;

        let reader = wad_loader.get_wad_zstd_entry_reader(&entry)?;

        let map_geo = LeagueMapGeo::read(&mut NoSeek::new(reader))?;

        let map_materials_bin_path = format!("data/maps/mapgeometry/map11/{}.materials.bin", map);

        let entry = wad_loader
            .wad
            .get_entry(LeagueLoader::hash_wad(&map_materials_bin_path))?;

        let mut reader = wad_loader.get_wad_zstd_entry_reader(&entry)?;

        let mut data = Vec::with_capacity(entry.target_size as usize);

        reader.read_to_end(&mut data)?;

        let materials_bin = PropFile::read(&mut Cursor::new(data))?;

        Ok(LeagueWadMapLoader {
            wad_loader,
            map_geo,
            materials_bin,
        })
    }

    pub async fn save_mapgeo(&self) -> Result<Vec<ConfigGeometryObject>, LeagueLoaderError> {
        let mut geometry_objects = Vec::new();

        for (i, map_mesh) in self.map_geo.meshes.iter().enumerate() {
            if map_mesh.layer_transition_behavior != LayerTransitionBehavior::Unaffected {
                continue;
            }

            let (all_positions, all_normals, all_uvs) = parse_vertex_data(&self.map_geo, map_mesh);

            for (j, submesh) in map_mesh.submeshes.iter().enumerate() {
                let intermediate_meshes = submesh_to_intermediate(
                    &submesh,
                    &self.map_geo,
                    map_mesh,
                    &all_positions,
                    &all_normals,
                    &all_uvs,
                );
                let material = self
                    .load_image_for_submesh(&submesh.material_name.text)
                    .unwrap();

                let mesh_path = format!("mapgeo/meshes/{}_{}.mesh", i, j);
                let mut file = get_asset_writer(&mesh_path).await?;
                let mut buffer = Vec::new();
                intermediate_meshes.write(&mut Cursor::new(&mut buffer))?;
                file.write_all(&buffer).await?;
                file.flush().await?;

                let material_path = get_bin_path(&submesh.material_name.text.clone());
                save_struct_to_file(&material_path, &material).await?;

                let texture_path = material.texture_path.clone();
                self.wad_loader
                    .save_wad_entry_to_file(&texture_path)
                    .await?;

                geometry_objects.push(ConfigGeometryObject {
                    mesh_path,
                    material_path,
                });
            }
        }

        Ok(geometry_objects)
    }

    pub fn load_image_for_submesh(&self, material_name: &str) -> Option<LeagueMaterial> {
        // 1. 根据材质名查找 texturePath

        let entry = self
            .materials_bin
            .entries
            .iter()
            .find(|v| v.hash == LeagueLoader::hash_bin(material_name))?;

        let material = from_entry::<StaticMaterialDef>(entry);

        // 2. 将列表转换为可迭代的 BinEmbed
        let embedded_samplers = material.sampler_values;

        // 3. 遍历所有 sampler，查找第一个包含 "texturePath" 的
        // `find_map` 会在找到第一个 Some(T) 后立即停止，比 filter_map + collect + first 更高效
        let texture_path = embedded_samplers.into_iter().find_map(|sampler_item| {
            let texture_name = &sampler_item.texture_name;
            if !(texture_name == "DiffuseTexture" || texture_name == "Diffuse_Texture") {
                return None;
            }
            Some(sampler_item.texture_path)
        });

        if let Some(texture_path) = texture_path {
            return Some(LeagueMaterial {
                texture_path: texture_path,
            });
        }

        None
    }

    pub async fn save_config_map(&self) -> Result<ConfigMap, LeagueLoaderError> {
        // 并行处理地图网格
        let geometry_objects = Vec::new();
        // self.save_mapgeo().await?;

        let mut minion_paths = HashMap::new();
        let mut barracks = HashMap::new();
        let mut characters = HashMap::new();
        let mut barrack_configs = HashMap::new();
        let mut environment_objects = HashMap::new();
        let mut skins = HashMap::new();
        let mut character_records = HashMap::new();

        for entry in self
            .materials_bin
            .iter_entry_by_class(LeagueLoader::hash_bin("BarracksConfig"))
        {
            barrack_configs
                .entry(entry.hash)
                .or_insert(from_entry::<BarracksConfig>(entry));
        }

        for entry in self.materials_bin.iter_entry_by_class(0x9d9f60d2) {
            let record = from_entry::<Unk0x9d9f60d2>(entry);
            characters.entry(entry.hash).or_insert(record);
        }

        for entry in self
            .materials_bin
            .iter_entry_by_class(LeagueLoader::hash_bin("MapPlaceableContainer"))
        {
            let map_placeable_container = from_entry::<MapPlaceableContainer>(entry);

            for (hash, value) in map_placeable_container.items {
                match value {
                    MapPlaceableContainerItems::Unk0x3c2bf0c0(unk0x3c2bf0c0) => {
                        environment_objects.entry(hash).or_insert(unk0x3c2bf0c0);
                    }
                    MapPlaceableContainerItems::Unk0x3c995caf(unk0x3c995caf) => {
                        let lane = match unk0x3c995caf.name.as_str() {
                            "MinionPath_Top" => Lane::Top,
                            "MinionPath_Mid" => Lane::Mid,
                            "MinionPath_Bot" => Lane::Bot,
                            _ => panic!("未知的小兵路径: {}", unk0x3c995caf.name),
                        };

                        let path = unk0x3c995caf.segments.iter().map(|v| v.xz()).collect();

                        minion_paths.entry(lane).or_insert(path);
                    }
                    MapPlaceableContainerItems::Unk0xc71ee7fb(unk0xc71ee7fb) => {
                        barracks.entry(hash).or_insert(unk0xc71ee7fb);
                    }
                    _ => {}
                }
            }
        }

        for environment_object in environment_objects.values() {
            let skin_key = environment_object.definition.skin.clone();
            let skin = self.wad_loader.save_environment_object(&skin_key).await?;
            skins.entry(skin_key).or_insert(skin);

            let character_record_key = environment_object.definition.character_record.clone();
            let character_record = self.wad_loader.load_character_record(&character_record_key);
            character_records
                .entry(character_record_key)
                .or_insert(character_record);
        }

        for character in characters.values() {
            let skin_key = character.skin.clone();
            let skin = self.wad_loader.save_environment_object(&skin_key).await?;
            skins.entry(skin_key).or_insert(skin);

            let character_record_key = character.character_record.clone();
            let character_record = self.wad_loader.load_character_record(&character_record_key);
            character_records
                .entry(character_record_key)
                .or_insert(character_record);
        }

        let configs = ConfigMap {
            geometry_objects,
            minion_paths,
            barracks,
            characters,
            barrack_configs,
            environment_objects,
            skins,
            character_records,
        };

        // 保存最终配置文件
        let path = get_bin_path(CONFIG_PATH_MAP);
        save_struct_to_file(&path, &configs).await?;

        Ok(configs)
    }

    pub async fn save_navigation_grid(&self) -> Result<ConfigNavigationGrid, LeagueLoaderError> {
        let nav_grid = self.load_navigation_grid().await?;
        let path = get_bin_path(CONFIG_PATH_MAP_NAV_GRID);
        save_struct_to_file(&path, &nav_grid).await?;
        Ok(nav_grid)
    }

    pub async fn load_navigation_grid(&self) -> Result<ConfigNavigationGrid, LeagueLoaderError> {
        let entry = self
            .materials_bin
            .iter_entry_by_class(LeagueLoader::hash_bin("MapContainer"))
            .next()
            .unwrap();

        let map_container = from_entry::<MapContainer>(entry);

        let components = map_container.components;

        println!("components: {:?}", components);

        let map_nav_grid = components
            .iter()
            .filter_map(|v| match v {
                MapContainerComponents::MapNavGrid(v) => Some(v),
                _ => None,
            })
            .next()
            .unwrap();

        let mut reader = self
            .wad_loader
            .get_wad_entry_reader_by_path(&map_nav_grid.nav_grid_path)
            .unwrap();

        let nav_grid = AiMeshNGrid::read(&mut reader).unwrap();

        let min_bounds = nav_grid.header.min_bounds.0.xz();

        let min_position = vec2(
            min_bounds.x,
            -(min_bounds.y + nav_grid.header.z_cell_count as f32 * nav_grid.header.cell_size),
        );

        let max_position = vec2(
            min_bounds.x + nav_grid.header.x_cell_count as f32 * nav_grid.header.cell_size,
            -min_bounds.y,
        );

        println!("min_position: {:?}", min_position);
        println!("max_position: {:?}", max_position);

        let width = max_position.x - min_position.x;
        let height = max_position.y - min_position.y;

        let cell_size = nav_grid.header.cell_size;

        println!(
            "width: {:?}, x len: {:?}",
            width,
            (width + 50.0) / cell_size
        );

        println!(
            "height: {:?}, y len: {:?}",
            height,
            (height + 50.0) / cell_size
        );

        let x_len = nav_grid.header.x_cell_count as usize;
        let y_len = nav_grid.header.z_cell_count as usize;

        println!("x_len: {:?}", x_len);
        println!("y_len: {:?}", y_len);

        let mut cells: Vec<ConfigNavigationGridCell> = Vec::new();

        for (i, cell) in nav_grid.navigation_grid.iter().enumerate() {
            let cell = ConfigNavigationGridCell {
                heuristic: cell.heuristic,
                vision_pathing_flags: nav_grid.vision_pathing_flags[i],
                river_region_flags: nav_grid.other_flags[i].river_region_flags,
                jungle_quadrant_flags: nav_grid.other_flags[i].jungle_quadrant_flags,
                main_region_flags: nav_grid.other_flags[i].main_region_flags,
                nearest_lane_flags: nav_grid.other_flags[i].nearest_lane_flags,
                poi_flags: nav_grid.other_flags[i].poi_flags,
                ring_flags: nav_grid.other_flags[i].ring_flags,
                srx_flags: nav_grid.other_flags[i].srx_flags,
            };

            cells.push(cell);
        }

        Ok(ConfigNavigationGrid {
            min_position,
            cell_size,
            x_len,
            y_len,
            cells: cells
                .chunks(x_len)
                // .map(|v| v.to_vec().into_iter().rev().collect())
                .map(|v| v.to_vec())
                .rev()
                .collect(),
            height_x_len: nav_grid.height_samples.x_count as usize,
            height_y_len: nav_grid.height_samples.z_count as usize,
            height_samples: nav_grid
                .height_samples
                .samples
                .chunks(nav_grid.height_samples.x_count as usize)
                .map(|v| v.to_vec())
                .rev()
                // .map(|v| v.to_vec().into_iter().rev().collect())
                .collect(),
        })
    }
}
