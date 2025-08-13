use std::io::BufReader;
#[cfg(unix)]
use std::os::unix::fs::FileExt;
#[cfg(windows)]
use std::os::windows::fs::FileExt;
use std::{
    collections::HashMap,
    fs::File,
    hash::Hasher,
    io::{self, Cursor, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::league::{
    neg_mat_z, skinned_mesh_to_intermediate, submesh_to_intermediate, AiMeshNGrid,
    AnimationGraphData, LeagueBinCharacterRecord, LeagueBinMaybeCharacterMapRecord, LeagueMaterial,
    LeagueMinionPath, LeagueSkeleton, LeagueSkinnedMesh, LeagueSkinnedMeshInternal,
    SkinCharacterDataProperties,
};
use crate::league::{
    static_conversion::parse_vertex_data, LayerTransitionBehavior, LeagueMapGeo, LeagueTexture,
};
use crate::{
    core::{
        ConfigEnvironmentObject, ConfigGeometryObject, ConfigJoint, ConfigNavigationGrid,
        ConfigNavigationGridCell, ConfigSkinnedMeshInverseBindposes, Configs, Health, Lane, Team,
    },
    entities::Barrack,
};

use bevy::image::TextureError;
use bevy::transform::components::Transform;
use binrw::BinWrite;
use binrw::{args, binread, io::NoSeek, BinRead, Endian};
use cdragon_prop::{
    BinEmbed, BinEntry, BinHash, BinList, BinMap, BinString, BinStruct, PropError, PropFile,
};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{fs::File as AsyncFile, io::AsyncWriteExt};
use twox_hash::XxHash64;
use zstd::Decoder;

#[derive(thiserror::Error, Debug)]
pub enum LeagueLoaderError {
    #[error("Could not load mesh: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not load texture: {0}")]
    BinRW(#[from] binrw::Error),

    #[error("Could not load texture: {0}")]
    Texture(#[from] TextureError),

    #[error("Could not load prop file: {0}")]
    PropError(#[from] PropError),

    #[error("Could not serialize: {0}")]
    Bincode(#[from] bincode::Error),
}

#[binread]
#[derive(Debug)]
#[br(magic = b"RW")]
#[br(little)]
pub struct LeagueWad {
    pub major: u8,
    pub minor: u8,

    #[br(pad_before = 0x108)]
    pub entry_count: u32,

    #[br(count = entry_count)]
    #[br(map = |entries: Vec<LeagueWadEntry>| {
        entries
            .into_iter()
            .map(|entry| (entry.path_hash, entry))
            .collect()
    })]
    pub entries: HashMap<u64, LeagueWadEntry>,
}

impl LeagueWad {
    #[inline]
    pub fn get_entry(&self, hash: u64) -> Option<&LeagueWadEntry> {
        self.entries.get(&hash)
    }
}

#[binread]
#[derive(Debug, Clone, Copy)]
#[br(little)]
pub struct LeagueWadEntry {
    pub path_hash: u64,
    pub offset: u32,
    pub size: u32,
    pub target_size: u32,

    #[br(map = |v: u8| parse_wad_data_format(v))]
    pub format: WadDataFormat,

    #[br(map = |v: u8| v != 0)]
    pub duplicate: bool,

    pub first_subchunk_index: u16,
    pub data_hash: u64,
}

#[binread]
#[derive(Debug, Clone, Copy)]
#[br(little)]
pub enum WadDataFormat {
    Uncompressed,
    Gzip,
    Redirection,
    Zstd,
    Chunked(u8),
}

#[inline]
fn parse_wad_data_format(format: u8) -> WadDataFormat {
    match format {
        0 => WadDataFormat::Uncompressed,
        1 => WadDataFormat::Gzip,
        2 => WadDataFormat::Redirection,
        3 => WadDataFormat::Zstd,
        b if b & 0xf == 4 => WadDataFormat::Chunked(b >> 4),
        _ => panic!("Invalid WadDataFormat: {}", format),
    }
}

#[binread]
#[derive(Debug)]
#[br(little)]
#[br(import { count: u32 })]
pub struct LeagueWadSubchunk {
    #[br(count = count)]
    pub chunks: Vec<LeagueWadSubchunkItem>,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct LeagueWadSubchunkItem {
    pub size: u32,
    pub target_size: u32,
    pub data_hash: u64,
}

pub struct LeagueLoader {
    pub root_dir: PathBuf,
    pub map_path: PathBuf,

    pub wad_file: Arc<File>,
    pub sub_chunk: LeagueWadSubchunk,

    pub wad: LeagueWad,
    pub mapgeo: LeagueMapGeo,
    pub materials_bin: PropFile,
}

pub struct ArcFileReader {
    file: Arc<File>,
    start_offset: u64,
    current_pos: u64,
}

impl ArcFileReader {
    #[inline]
    pub fn new(file: Arc<File>, start_offset: u64) -> Self {
        Self {
            file,
            start_offset,
            current_pos: 0,
        }
    }
}

impl Read for ArcFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let absolute_offset = self.start_offset + self.current_pos;

        #[cfg(unix)]
        let bytes_read = self.file.read_at(buf, absolute_offset)?;
        #[cfg(windows)]
        let bytes_read = self.file.seek_read(buf, absolute_offset)?;

        self.current_pos += bytes_read as u64;

        Ok(bytes_read)
    }
}

impl Seek for ArcFileReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(p) => p as i64,
            SeekFrom::End(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "SeekFrom::End is not supported",
                ));
            }
            SeekFrom::Current(p) => self.current_pos as i64 + p,
        };

        if new_pos < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid seek to a negative position",
            ));
        }

        self.current_pos = new_pos as u64;
        Ok(self.current_pos)
    }
}

impl LeagueLoader {
    pub fn new(
        root_dir: &str,
        map_path: &str,
        map_geo_path: &str,
    ) -> Result<LeagueLoader, LeagueLoaderError> {
        let root_dir = Path::new(root_dir);
        let map_relative_path = Path::new(map_path);
        let map_absolute_path = root_dir.join(map_relative_path);

        let file = Arc::new(File::open(&map_absolute_path)?);
        let wad = LeagueWad::read(&mut ArcFileReader::new(file.clone(), 0))?;

        let sub_chunk = Self::load_subchunk(&wad, &file, map_path)?;

        let map_geo = Self::load_map_geo(&wad, &file, map_geo_path)?;

        let map_materials = Self::load_map_materials(&wad, &file, map_geo_path)?;

        Ok(LeagueLoader {
            root_dir: root_dir.to_path_buf(),
            map_path: map_relative_path.to_path_buf(),
            wad_file: file,
            sub_chunk,
            wad,
            mapgeo: map_geo,
            materials_bin: map_materials,
        })
    }

    pub async fn save_configs(&self) -> Result<Configs, LeagueLoaderError> {
        // 并行处理地图网格
        let geometry_objects = self.save_mapgeo().await?;

        let (environment_objects, barracks) = self.save_environment_objects().await?;

        // 处理小兵路径（这部分不涉及文件 I/O，保持同步）
        let minion_paths: Vec<LeagueMinionPath> = self
            .materials_bin
            .entries
            .iter()
            .filter(|v| v.ctype.hash == Self::hash_bin("MapPlaceableContainer"))
            .map(|v| v.getv::<BinMap>(Self::hash_bin("items").into()).unwrap())
            .flat_map(|v| v.downcast::<BinHash, BinStruct>().unwrap())
            .filter(|v| v.1.ctype.hash == 0x3c995caf)
            .map(|v| LeagueMinionPath::from(&v.1))
            .collect();

        let minion_paths = minion_paths
            .into_iter()
            .map(|v| (v.lane, v.path))
            .collect::<HashMap<_, _>>();

        let configs = Configs {
            geometry_objects,
            environment_objects,
            minion_paths,
            barracks,
            navigation_grid: self.load_navigation_grid().await?,
        };

        // 保存最终配置文件
        let path = get_bin_path("configs");
        save_struct_to_file(&path, &configs).await?;

        Ok(configs)
    }

    pub async fn save_mapgeo(&self) -> Result<Vec<ConfigGeometryObject>, LeagueLoaderError> {
        let mut geometry_objects = Vec::new();

        for (i, map_mesh) in self.mapgeo.meshes.iter().enumerate() {
            if map_mesh.layer_transition_behavior != LayerTransitionBehavior::Unaffected {
                continue;
            }

            let (all_positions, all_normals, all_uvs) = parse_vertex_data(&self.mapgeo, map_mesh);

            for (j, submesh) in map_mesh.submeshes.iter().enumerate() {
                let intermediate_meshes = submesh_to_intermediate(
                    &submesh,
                    &self.mapgeo,
                    map_mesh,
                    &all_positions,
                    &all_normals,
                    &all_uvs,
                );
                let material = self
                    .find_and_load_image_for_submesh(&submesh.material_name.text)
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
                self.save_wad_entry_to_file(&texture_path).await?;

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
            Vec<(Transform, ConfigEnvironmentObject, Option<Health>)>,
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
            .filter(|v| v.ctype.hash == Self::hash_bin("MapPlaceableContainer"))
            .filter_map(|v| v.getv::<BinMap>(Self::hash_bin("items").into()))
            .filter_map(|v| v.downcast::<BinHash, BinStruct>())
            .flatten()
        {
            match entry.1.ctype.hash {
                0x1e1cce2 => {
                    let mut character_map_record = LeagueBinMaybeCharacterMapRecord::from(&entry.1);
                    neg_mat_z(&mut character_map_record.transform);
                    let character_record = self
                        .load_character_record(&character_map_record.definition.character_record);
                    let transform = Transform::from_matrix(character_map_record.transform);
                    let skin = character_map_record.definition.skin;

                    let environment_object = self.save_environment_object(&skin).await?;

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

    pub async fn load_navigation_grid(&self) -> Result<ConfigNavigationGrid, LeagueLoaderError> {
        let map_container = self
            .materials_bin
            .entries
            .iter()
            .find(|v| v.ctype.hash == Self::hash_bin("MapContainer"))
            .unwrap();

        let components = map_container
            .getv::<BinList>(Self::hash_bin("components").into())
            .unwrap();

        let map_nav_grid = components
            .downcast::<BinStruct>()
            .unwrap()
            .iter()
            .find(|v| v.ctype.hash == Self::hash_bin("MapNavGrid"))
            .unwrap();

        let nav_grid_path = map_nav_grid
            .getv::<BinString>(Self::hash_bin("NavGridPath").into())
            .unwrap()
            .0
            .clone();

        let mut reader = self.get_wad_entry_reader_by_path(&nav_grid_path).unwrap();

        let nav_grid = AiMeshNGrid::read(&mut reader).unwrap();

        let min_grid_pos = nav_grid.header.min_bounds.0;
        let cell_size = nav_grid.header.cell_size;
        let x_len = nav_grid.header.x_cell_count as usize;
        let y_len = nav_grid.header.z_cell_count as usize;

        let mut cells: Vec<ConfigNavigationGridCell> = Vec::new();

        for (i, cell) in nav_grid.navigation_grid.iter().enumerate() {
            cells.push(ConfigNavigationGridCell {
                y: cell.center_height,
                heuristic: cell.heuristic,
                vision_pathing_flags: nav_grid.vision_pathing_flags[i],
                river_region_flags: nav_grid.other_flags[i].river_region_flags,
                jungle_quadrant_flags: nav_grid.other_flags[i].jungle_quadrant_flags,
                main_region_flags: nav_grid.other_flags[i].main_region_flags,
                nearest_lane_flags: nav_grid.other_flags[i].nearest_lane_flags,
                poi_flags: nav_grid.other_flags[i].poi_flags,
                ring_flags: nav_grid.other_flags[i].ring_flags,
                srx_flags: nav_grid.other_flags[i].srx_flags,
            });
        }

        let cells = cells.chunks(x_len).map(|v| v.to_vec()).collect();

        Ok(ConfigNavigationGrid {
            min_grid_pos,
            cell_size,
            x_len,
            y_len,
            cells,
        })
    }

    pub fn find_and_load_image_for_submesh(&self, material_name: &str) -> Option<LeagueMaterial> {
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

    pub fn load_character_record(&self, character_record: &str) -> LeagueBinCharacterRecord {
        let name = character_record.split("/").nth(1).unwrap();

        let path = format!("data/characters/{0}/{0}.bin", name);

        let character_bin = self.get_prop_bin_by_path(&path).unwrap();

        let character_record = character_bin
            .entries
            .iter()
            .find(|v| v.path.hash == Self::hash_bin(&character_record))
            .unwrap();

        let character_record = character_record.into();

        character_record
    }

    pub async fn save_environment_object(
        &self,
        skin: &str,
    ) -> Result<ConfigEnvironmentObject, LeagueLoaderError> {
        let (skin_character_data_properties, flat_map) = self.load_character_skin(&skin);

        let texture_path = skin_character_data_properties
            .skin_mesh_properties
            .texture
            .clone();
        self.save_wad_entry_to_file(&texture_path).await?;

        let material = LeagueMaterial {
            texture_path: texture_path.clone(),
        };

        let skeleton_path = skin_character_data_properties
            .skin_mesh_properties
            .skeleton
            .clone();
        self.save_wad_entry_to_file(&skeleton_path).await?;

        let mut reader = self
            .get_wad_entry_no_seek_reader_by_path(
                &skin_character_data_properties
                    .skin_mesh_properties
                    .simple_skin,
            )
            .unwrap();

        let league_skinned_mesh =
            LeagueSkinnedMesh::from(LeagueSkinnedMeshInternal::read(&mut reader).unwrap());

        let league_skeleton = self
            .get_wad_entry_reader_by_path(&skeleton_path)
            .map(|mut v| LeagueSkeleton::read(&mut v).unwrap())
            .unwrap();

        let animation_graph_data: AnimationGraphData = flat_map
            .get(
                &skin_character_data_properties
                    .skin_animation_properties
                    .animation_graph_data,
            )
            .unwrap()
            .into();

        let clip_map = animation_graph_data
            .clip_data_map
            .into_iter()
            .map(|(k, v)| (k, v.animation_resource_data.animation_file_path))
            .collect::<HashMap<_, _>>();

        // 保存动画文件
        for (_, clip_path) in &clip_map {
            self.save_wad_entry_to_file(clip_path).await?;
        }

        let material_path = get_bin_path(&format!("ASSETS/{}/material", skin));
        save_struct_to_file(&material_path, &material).await?;

        let mut submesh_paths = Vec::new();

        for (i, range) in league_skinned_mesh.ranges.iter().enumerate() {
            let mesh = skinned_mesh_to_intermediate(&league_skinned_mesh, i).unwrap();
            let mesh_path = format!("ASSETS/{}/meshes/{}.mesh", skin, &range.name);

            let mut file = get_asset_writer(&mesh_path).await?;
            let mut buffer = Vec::new();
            mesh.write(&mut Cursor::new(&mut buffer))
                .map_err(|e| LeagueLoaderError::Io(io::Error::new(io::ErrorKind::Other, e)))?;
            file.write_all(&buffer).await?;
            file.flush().await?;

            submesh_paths.push(mesh_path);
        }

        let inverse_bind_poses = league_skeleton
            .modern_data
            .influences
            .iter()
            .map(|&v| league_skeleton.modern_data.joints[v as usize].inverse_bind_transform)
            .collect::<Vec<_>>();

        let inverse_bind_pose_path = get_bin_path(&format!("ASSETS/{}/inverse_bind_pose", skin));
        save_struct_to_file(
            &inverse_bind_pose_path,
            &ConfigSkinnedMeshInverseBindposes {
                inverse_bindposes: inverse_bind_poses,
            },
        )
        .await?;

        Ok(ConfigEnvironmentObject {
            skin_scale: skin_character_data_properties
                .skin_mesh_properties
                .skin_scale,
            material_path,
            submesh_paths,
            joint_influences_indices: league_skeleton.modern_data.influences,
            inverse_bind_pose_path,
            joints: league_skeleton
                .modern_data
                .joints
                .iter()
                .map(|joint| ConfigJoint {
                    hash: Self::hash_joint(&joint.name),
                    transform: Transform::from_matrix(joint.local_transform),
                    parent_index: joint.parent_index,
                })
                .collect(),
            clip_map,
        })
    }

    pub fn load_character_skin(
        &self,
        skin: &str,
    ) -> (SkinCharacterDataProperties, HashMap<u32, BinEntry>) {
        let skin_path = format!("data/{}.bin", skin);

        let skin_bin = self.get_prop_bin_by_path(&skin_path).unwrap();

        let skin_mesh_properties = skin_bin
            .entries
            .iter()
            .find(|v| v.ctype.hash == Self::hash_bin("SkinCharacterDataProperties"))
            .unwrap();

        let flat_map: HashMap<_, _> = skin_bin
            .linked_files
            .iter()
            .map(|v| self.get_prop_bin_by_path(v).unwrap())
            .flat_map(|v| v.entries)
            .map(|v| (v.path.hash, v))
            .collect();

        (skin_mesh_properties.into(), flat_map)
    }

    fn load_subchunk(
        wad: &LeagueWad,
        file: &Arc<File>,
        map_path: &str,
    ) -> Result<LeagueWadSubchunk, LeagueLoaderError> {
        let entry = Self::get_wad_subchunk_entry(wad, map_path);
        let reader = Self::get_wad_zstd_entry_reader(file, &entry)?;

        Ok(LeagueWadSubchunk::read_options(
            &mut NoSeek::new(reader),
            Endian::Little,
            args! { count: entry.target_size / 16 },
        )?)
    }

    fn load_map_geo(
        wad: &LeagueWad,
        file: &Arc<File>,
        map_geo_path: &str,
    ) -> Result<LeagueMapGeo, LeagueLoaderError> {
        let entry = Self::get_wad_entry(wad, Self::compute_wad_hash(map_geo_path));
        let reader = Self::get_wad_zstd_entry_reader(file, &entry)?;

        Ok(LeagueMapGeo::read(&mut NoSeek::new(reader))?)
    }

    fn load_map_materials(
        wad: &LeagueWad,
        file: &Arc<File>,
        map_geo_path: &str,
    ) -> Result<PropFile, LeagueLoaderError> {
        let map_materials_path = map_geo_path
            .replace(".mapgeo", ".materials.bin")
            .replace('\\', "/")
            .to_lowercase();

        let entry = Self::get_wad_entry(wad, Self::compute_wad_hash(&map_materials_path));
        let mut reader = Self::get_wad_zstd_entry_reader(file, &entry)?;

        let mut data = Vec::with_capacity(entry.target_size as usize);
        reader.read_to_end(&mut data)?;

        Ok(PropFile::from_slice(&data)?)
    }

    #[inline]
    pub fn get_wad_entry_by_hash(&self, hash: u64) -> Option<LeagueWadEntry> {
        self.wad.get_entry(hash).copied()
    }

    #[inline]
    pub fn get_wad_entry_by_path(&self, path: &str) -> Option<LeagueWadEntry> {
        self.get_wad_entry_by_hash(Self::compute_wad_hash(&path.to_lowercase()))
    }

    pub fn get_wad_entry_reader(
        &self,
        entry: &LeagueWadEntry,
    ) -> Result<Box<dyn Read + '_>, LeagueLoaderError> {
        match entry.format {
            WadDataFormat::Uncompressed => Ok(Box::new(
                ArcFileReader::new(self.wad_file.clone(), entry.offset as u64)
                    .take(entry.size as u64),
            )),
            WadDataFormat::Redirection | WadDataFormat::Gzip => {
                panic!("wad entry format not supported")
            }
            WadDataFormat::Zstd => Self::get_wad_zstd_entry_reader(&self.wad_file, entry),
            WadDataFormat::Chunked(subchunk_count) => {
                self.read_chunked_entry(entry, subchunk_count)
            }
        }
    }

    fn read_chunked_entry(
        &self,
        entry: &LeagueWadEntry,
        subchunk_count: u8,
    ) -> Result<Box<dyn Read + '_>, LeagueLoaderError> {
        if self.sub_chunk.chunks.is_empty() {
            panic!("No subchunk data available");
        }

        let mut offset = 0u64;
        let mut result = Vec::with_capacity(entry.target_size as usize);

        for i in 0..subchunk_count {
            let chunk_index = (entry.first_subchunk_index as usize) + (i as usize);
            if chunk_index >= self.sub_chunk.chunks.len() {
                panic!("Subchunk index out of bounds");
            }

            let subchunk_entry = &self.sub_chunk.chunks[chunk_index];
            let mut subchunk_reader =
                ArcFileReader::new(self.wad_file.clone(), entry.offset as u64 + offset)
                    .take(subchunk_entry.size as u64);

            offset += subchunk_entry.size as u64;

            if subchunk_entry.size == subchunk_entry.target_size {
                subchunk_reader.read_to_end(&mut result)?;
            } else {
                zstd::stream::read::Decoder::new(subchunk_reader)?.read_to_end(&mut result)?;
            }
        }

        Ok(Box::new(Cursor::new(result)))
    }

    pub fn get_wad_entry_no_seek_reader_by_hash(
        &self,
        hash: u64,
    ) -> Result<NoSeek<Box<dyn Read + '_>>, LeagueLoaderError> {
        let entry = self
            .get_wad_entry_by_hash(hash)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "WAD entry not found"))?;
        self.get_wad_entry_reader(&entry).map(|v| NoSeek::new(v))
    }

    pub fn get_wad_entry_no_seek_reader_by_path(
        &self,
        path: &str,
    ) -> Result<NoSeek<Box<dyn Read + '_>>, LeagueLoaderError> {
        self.get_wad_entry_no_seek_reader_by_hash(Self::compute_wad_hash(path))
    }

    pub fn get_wad_entry_reader_by_path(
        &self,
        path: &str,
    ) -> Result<BufReader<Cursor<Vec<u8>>>, LeagueLoaderError> {
        self.get_wad_entry_buffer_by_path(path)
            .map(|v| BufReader::new(Cursor::new(v)))
    }

    pub fn get_wad_entry_buffer_by_path(&self, path: &str) -> Result<Vec<u8>, LeagueLoaderError> {
        let entry = self
            .get_wad_entry_by_path(path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "WAD entry not found"))?;
        let mut buf = Vec::new();
        self.get_wad_entry_reader(&entry).map(|mut v| {
            v.read_to_end(&mut buf).unwrap();
            buf
        })
    }

    pub fn get_texture_by_hash(&self, hash: u64) -> Result<LeagueTexture, LeagueLoaderError> {
        let mut reader = self.get_wad_entry_no_seek_reader_by_hash(hash)?;
        Ok(LeagueTexture::read(&mut reader)?)
    }

    pub fn get_texture_by_path(&self, path: &str) -> Result<LeagueTexture, LeagueLoaderError> {
        let mut reader = self.get_wad_entry_no_seek_reader_by_path(path)?;
        Ok(LeagueTexture::read(&mut reader)?)
    }

    pub fn get_prop_bin_by_path(&self, path: &str) -> Result<PropFile, LeagueLoaderError> {
        let mut reader = self.get_wad_entry_no_seek_reader_by_path(path)?;
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(PropFile::from_slice(&data)?)
    }

    #[inline]
    pub fn compute_wad_hash(s: &str) -> u64 {
        let mut h = XxHash64::with_seed(0);
        h.write(s.to_ascii_lowercase().as_bytes());
        h.finish()
    }

    #[inline]
    pub fn hash_bin(s: &str) -> u32 {
        s.to_ascii_lowercase().bytes().fold(0x811c9dc5_u32, |h, b| {
            (h ^ b as u32).wrapping_mul(0x01000193)
        })
    }

    pub fn hash_joint(s: &str) -> u32 {
        let mut hash = 0u32;
        for b in s.to_ascii_lowercase().bytes() {
            hash = (hash << 4) + (b as u32);

            let high = hash & 0xf0000000;

            if high != 0 {
                hash ^= high >> 24;
            }

            hash &= !high;
        }
        hash
    }

    #[inline]
    fn get_wad_entry(wad: &LeagueWad, hash: u64) -> LeagueWadEntry {
        *wad.entries.get(&hash).expect("WAD entry not found")
    }

    fn get_wad_subchunk_entry(wad: &LeagueWad, map_path: &str) -> LeagueWadEntry {
        let subchunk_path = map_path
            .replace(".client", ".subchunktoc")
            .replace('\\', "/")
            .to_lowercase();

        Self::get_wad_entry(wad, Self::compute_wad_hash(&subchunk_path))
    }

    fn get_wad_zstd_entry_reader(
        wad_file: &Arc<File>,
        entry: &LeagueWadEntry,
    ) -> Result<Box<dyn Read>, LeagueLoaderError> {
        let reader =
            ArcFileReader::new(wad_file.clone(), entry.offset as u64).take(entry.size as u64);
        let decoder = Decoder::new(reader)?;
        Ok(Box::new(decoder))
    }

    pub async fn save_wad_entry_to_file(&self, path: &str) -> Result<(), LeagueLoaderError> {
        let buffer = self.get_wad_entry_buffer_by_path(path)?;
        let mut file = get_asset_writer(&path).await?;
        file.write_all(&buffer).await?;
        file.flush().await?;
        Ok(())
    }
}

fn ensure_dir_exists(path: &str) -> Result<(), LeagueLoaderError> {
    let dir = Path::new(path).parent().unwrap();
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

pub async fn save_struct_to_file<T: Serialize>(
    path: &str,
    data: &T,
) -> Result<(), LeagueLoaderError> {
    let serialized = bincode::serialize(data)?;
    let mut file = get_asset_writer(path).await?;
    file.write_all(&serialized).await?;
    file.flush().await?;
    Ok(())
}

pub async fn get_asset_writer(path: &str) -> Result<AsyncFile, LeagueLoaderError> {
    let path = format!("assets/{}", path);
    println!("√ {}", path);
    ensure_dir_exists(&path)?;
    let file = AsyncFile::create(path).await?;
    Ok(file)
}

pub fn get_bin_path(path: &str) -> String {
    format!("{}.bin", path)
}

pub fn get_struct_from_file<T: DeserializeOwned>(path: &str) -> Result<T, LeagueLoaderError> {
    let mut file = File::open(format!("assets/{}", &get_bin_path(path)))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let data = bincode::deserialize(&data)?;
    Ok(data)
}
