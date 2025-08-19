use std::{
    collections::HashMap,
    io::{Cursor, Read},
};

use bevy::{
    math::{vec2, Vec3Swizzles},
    transform::components::Transform,
};
use binrw::{io::NoSeek, BinRead, BinWrite};
use cdragon_prop::{BinEmbed, BinHash, BinList, BinMap, BinString, BinStruct, PropFile};
use tokio::io::AsyncWriteExt;

use crate::{
    core::CONFIG_PATH_MAP,
    league::{static_conversion::parse_vertex_data, LayerTransitionBehavior, LeagueMapGeo},
};
use crate::{
    core::CONFIG_PATH_MAP_NAV_GRID,
    league::{
        get_asset_writer, get_bin_path, neg_mat_z, save_struct_to_file, submesh_to_intermediate,
        AiMeshNGrid, LeagueBinMaybeCharacterMapRecord, LeagueLoader, LeagueLoaderError,
        LeagueMaterial, LeagueMinionPath, LeagueWadLoader,
    },
};
use crate::{
    core::{
        ConfigCharacterSkin, ConfigGeometryObject, ConfigMap, ConfigNavigationGrid,
        ConfigNavigationGridCell, Health, Lane, Team,
    },
    entities::Barrack,
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

        let materials_bin = PropFile::from_slice(&data)?;

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

    pub async fn save_environment_objects(
        &self,
    ) -> Result<
        (
            Vec<(Transform, ConfigCharacterSkin, Option<Health>)>,
            Vec<(Transform, Team, Lane, Barrack)>,
        ),
        LeagueLoaderError,
    > {
        let mut environment_objects = Vec::new();
        let mut barracks = Vec::new();

        // 处理环境对象和兵营
        for entry in self
            .materials_bin
            .entries
            .iter()
            .filter(|v| v.ctype.hash == LeagueLoader::hash_bin("MapPlaceableContainer"))
            .filter_map(|v| v.getv::<BinMap>(LeagueLoader::hash_bin("items").into()))
            .filter_map(|v| v.downcast::<BinHash, BinStruct>())
            .flatten()
        {
            match entry.1.ctype.hash {
                0x1e1cce2 => {
                    let character_map_record = LeagueBinMaybeCharacterMapRecord::from(&entry.1);

                    let character_record = self
                        .wad_loader
                        .load_character_record(&character_map_record.definition.character_record);
                    let transform =
                        Transform::from_matrix(neg_mat_z(&character_map_record.transform));
                    let skin = character_map_record.definition.skin;

                    let environment_object = self.wad_loader.save_environment_object(&skin).await?;

                    environment_objects.push((
                        transform,
                        environment_object,
                        character_record
                            .base_hp
                            .map(|v| Health { value: v, max: v }),
                    ));
                }
                0x71d0eabd => {
                    let barrack = self.save_barrack(&entry.1).await?;
                    barracks.push(barrack);
                }
                _ => {}
            }
        }

        Ok((environment_objects, barracks))
    }

    pub fn load_image_for_submesh(&self, material_name: &str) -> Option<LeagueMaterial> {
        // 1. 根据材质名查找 texturePath
        let binhash = LeagueLoader::hash_bin(material_name);

        for entry in &self.materials_bin.entries {
            if entry.path.hash == binhash {
                let sampler_values =
                    entry.getv::<BinList>(LeagueLoader::hash_bin("samplerValues").into())?;

                // 2. 将列表转换为可迭代的 BinEmbed
                let embedded_samplers = sampler_values.downcast::<BinEmbed>()?;

                // 3. 遍历所有 sampler，查找第一个包含 "texturePath" 的
                // `find_map` 会在找到第一个 Some(T) 后立即停止，比 filter_map + collect + first 更高效
                let texture_path = embedded_samplers.into_iter().find_map(|sampler_item| {
                    let texture_name = &sampler_item
                        .getv::<BinString>(LeagueLoader::hash_bin("textureName").into())?
                        .0;
                    if !(texture_name == "DiffuseTexture" || texture_name == "Diffuse_Texture") {
                        return None;
                    }
                    sampler_item
                        .getv::<BinString>(LeagueLoader::hash_bin("texturePath").into())
                        .map(|v| v.0.clone())
                });

                if let Some(texture_path) = texture_path {
                    return Some(LeagueMaterial {
                        texture_path: texture_path,
                    });
                }
            }
        }

        None
    }

    pub async fn save_config_map(&self) -> Result<ConfigMap, LeagueLoaderError> {
        // 并行处理地图网格
        let geometry_objects = self.save_mapgeo().await?;

        let (environment_objects, barracks) = self.save_environment_objects().await?;

        // 处理小兵路径（这部分不涉及文件 I/O，保持同步）
        let minion_paths: Vec<LeagueMinionPath> = self
            .materials_bin
            .entries
            .iter()
            .filter(|v| v.ctype.hash == LeagueLoader::hash_bin("MapPlaceableContainer"))
            .map(|v| {
                v.getv::<BinMap>(LeagueLoader::hash_bin("items").into())
                    .unwrap()
            })
            .flat_map(|v| v.downcast::<BinHash, BinStruct>().unwrap())
            .filter(|v| v.1.ctype.hash == 0x3c995caf)
            .map(|v| LeagueMinionPath::from(&v.1))
            .collect();

        let minion_paths = minion_paths
            .into_iter()
            .map(|v| (v.lane, v.path))
            .collect::<HashMap<_, _>>();

        let configs = ConfigMap {
            geometry_objects,
            environment_objects,
            minion_paths,
            barracks,
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
        let map_container = self
            .materials_bin
            .entries
            .iter()
            .find(|v| v.ctype.hash == LeagueLoader::hash_bin("MapContainer"))
            .unwrap();

        let components = map_container
            .getv::<BinList>(LeagueLoader::hash_bin("components").into())
            .unwrap();

        let map_nav_grid = components
            .downcast::<BinStruct>()
            .unwrap()
            .iter()
            .find(|v| v.ctype.hash == LeagueLoader::hash_bin("MapNavGrid"))
            .unwrap();

        let nav_grid_path = map_nav_grid
            .getv::<BinString>(LeagueLoader::hash_bin("NavGridPath").into())
            .unwrap()
            .0
            .clone();

        let mut reader = self
            .wad_loader
            .get_wad_entry_reader_by_path(&nav_grid_path)
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
