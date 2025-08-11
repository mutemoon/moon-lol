use crate::core::{
    ConfigAnimationGraph, ConfigEnvironmentObject, ConfigGeometryObject, ConfigJoint, Configs,
    Health,
};
use crate::league::{
    find_and_load_image_for_submesh, static_conversion::parse_vertex_data, LayerTransitionBehavior,
    LeagueMapGeo, LeagueTexture,
};
use crate::league::{
    neg_mat_z, skinned_mesh_to_intermediate, submesh_to_intermediate, AnimationClipData,
    AnimationGraphData, LeagueBinCharacterRecord, LeagueBinMaybeCharacterMapRecord,
    LeagueMinionPath, LeagueSkeleton, LeagueSkinnedMesh, LeagueSkinnedMeshInternal,
    SkinCharacterDataProperties,
};
use bevy::transform::components::Transform;
use bevy::{
    image::TextureError,
    scene::ron,
    scene::ron::{de::SpannedError, ser::to_writer_pretty},
};
use binrw::BinWrite;
use binrw::{args, binread, io::NoSeek, BinRead, Endian};
use cdragon_prop::{BinEntry, BinHash, BinMap, BinStruct, PropError, PropFile};
use serde::Serialize;
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
use tokio::{fs::File as AsyncFile, io::AsyncWriteExt};
use twox_hash::XxHash64;
use zstd::Decoder;

#[derive(thiserror::Error, Debug)]
pub enum LeagueLoaderError {
    #[error("Could not load mesh: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not load material: {0}")]
    RonSpanned(#[from] SpannedError),

    #[error("Could not load material: {0}")]
    RON(#[from] ron::Error),

    #[error("Could not load texture: {0}")]
    BinRW(#[from] binrw::Error),

    #[error("Could not load texture: {0}")]
    Texture(#[from] TextureError),

    #[error("Could not load prop file: {0}")]
    PropError(#[from] PropError),
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

    pub async fn save_configs_async(&self) -> Result<Configs, LeagueLoaderError> {
        let mut configs = Configs::default();

        // 并行处理地图网格
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
                let material = find_and_load_image_for_submesh(
                    &submesh.material_name.text,
                    &self.materials_bin,
                )
                .unwrap();

                let mesh_path = format!("mapgeo/meshes/{}_{}.mesh", i, j);
                let material_path = submesh.material_name.text.clone();
                let texture_path = material.texture_path.clone();

                // 分别保存，而不是在异步任务中
                save_struct_to_file_async(&material_path, &material).await?;
                self.save_wad_entry_to_file_async(&texture_path).await?;

                let mut file = get_async_asset_writer(&mesh_path).await?;
                let mut buffer = Vec::new();
                intermediate_meshes
                    .write(&mut Cursor::new(&mut buffer))
                    .map_err(|e| LeagueLoaderError::Io(io::Error::new(io::ErrorKind::Other, e)))?;
                file.write_all(&buffer).await?;
                file.flush().await?;

                configs.geometry_objects.push(ConfigGeometryObject {
                    mesh_path,
                    material_path,
                });
            }
        }

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
                    let mut character_map_record = LeagueBinMaybeCharacterMapRecord::from(&entry.1);
                    neg_mat_z(&mut character_map_record.transform);
                    let character_record = self
                        .load_character_record(&character_map_record.definition.character_record);
                    let transform = Transform::from_matrix(character_map_record.transform);
                    let skin = character_map_record.definition.skin;

                    let environment_object = self.save_environment_object_async(&skin).await?;

                    configs.environment_objects.push((
                        transform,
                        environment_object,
                        character_record
                            .base_hp
                            .map(|v| Health { value: v, max: v }),
                    ));
                }
                0x71d0eabd => {
                    let barrack = self.save_barrack_async(&entry.1).await?;
                    configs.barracks.push(barrack);
                }
                _ => {}
            }
        }

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

        for path in minion_paths {
            configs.minion_paths.insert(path.lane, path.path);
        }

        // 保存最终配置文件
        save_struct_to_file_async("configs.ron", &configs).await?;

        Ok(configs)
    }

    pub fn save_configs(&self) -> Configs {
        let mut configs = Configs::default();

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
                let material = find_and_load_image_for_submesh(
                    &submesh.material_name.text,
                    &self.materials_bin,
                )
                .unwrap();

                save_struct_to_file(&submesh.material_name.text, &material).unwrap();

                self.save_wad_entry_to_file(&material.texture_path).unwrap();

                let mesh_path = format!("mapgeo/meshes/{}_{}.mesh", i, j);

                intermediate_meshes
                    .write(&mut get_asset_writer(&mesh_path).unwrap())
                    .unwrap();

                configs.geometry_objects.push(ConfigGeometryObject {
                    mesh_path,
                    material_path: submesh.material_name.text.clone(),
                });
            }
        }

        self.materials_bin
            .entries
            .iter()
            .filter(|v| v.ctype.hash == LeagueLoader::hash_bin("MapPlaceableContainer"))
            .filter_map(|v| v.getv::<BinMap>(LeagueLoader::hash_bin("items").into()))
            .filter_map(|v| v.downcast::<BinHash, BinStruct>())
            .flatten()
            .for_each(|v| match v.1.ctype.hash {
                0x1e1cce2 => {
                    let mut character_map_record = LeagueBinMaybeCharacterMapRecord::from(&v.1);

                    neg_mat_z(&mut character_map_record.transform);

                    let character_record = self
                        .load_character_record(&character_map_record.definition.character_record);

                    let transform = Transform::from_matrix(character_map_record.transform);

                    let skin = character_map_record.definition.skin;

                    let environment_object = self.save_environment_object(&skin).unwrap();

                    configs.environment_objects.push((
                        transform,
                        environment_object,
                        character_record
                            .base_hp
                            .map(|v| Health { value: v, max: v }),
                    ));
                }
                0x71d0eabd => {
                    let barrack = self.save_barrack(&v.1).unwrap();
                    configs.barracks.push(barrack);
                }
                _ => {}
            });

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

        for path in minion_paths {
            configs.minion_paths.insert(path.lane, path.path);
        }

        to_writer_pretty(
            &mut get_asset_writer("configs.ron").unwrap(),
            &configs,
            Default::default(),
        )
        .unwrap();

        configs
    }

    pub fn load_character_record(&self, character_record: &str) -> LeagueBinCharacterRecord {
        let name = character_record.split("/").nth(1).unwrap();

        let path = format!("data/characters/{0}/{0}.bin", name);

        let character_bin = self.get_prop_bin_by_path(&path).unwrap();

        let character_record = character_bin
            .entries
            .iter()
            .find(|v| v.path.hash == LeagueLoader::hash_bin(&character_record))
            .unwrap();

        let character_record = character_record.into();

        character_record
    }

    pub fn save_environment_object(
        &self,
        skin: &str,
    ) -> Result<ConfigEnvironmentObject, LeagueLoaderError> {
        let (skin_character_data_properties, flat_map) = self.load_character_skin(&skin);

        let texture_path = skin_character_data_properties.skin_mesh_properties.texture;

        self.save_wad_entry_to_file(&texture_path).unwrap();

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
            .get_wad_entry_reader_by_path(
                &skin_character_data_properties.skin_mesh_properties.skeleton,
            )
            .map(|mut v| LeagueSkeleton::read(&mut v).unwrap())
            .unwrap();

        let skeleton_path = skin_character_data_properties.skin_mesh_properties.skeleton;

        self.save_wad_entry_to_file(&skeleton_path).unwrap();

        let animation_graph_data: AnimationGraphData = flat_map
            .get(
                &skin_character_data_properties
                    .skin_animation_properties
                    .animation_graph_data,
            )
            .unwrap()
            .into();

        let clip_paths = animation_graph_data
            .clip_data_map
            .iter()
            .filter_map(|(_, v)| match v {
                AnimationClipData::AtomicClipData {
                    animation_resource_data,
                } => Some(animation_resource_data.animation_file_path.clone()),
                AnimationClipData::Unknown => None,
            })
            .collect::<Vec<_>>();

        clip_paths.iter().for_each(|v| {
            self.save_wad_entry_to_file(v).unwrap();
        });

        let mut submesh_paths = Vec::new();

        for (i, range) in league_skinned_mesh.ranges.iter().enumerate() {
            let mesh = skinned_mesh_to_intermediate(&league_skinned_mesh, i).unwrap();

            let mesh_path = format!("skin_meshes/{}.mesh", &range.name);

            mesh.write(&mut get_asset_writer(&mesh_path).unwrap())
                .unwrap();

            submesh_paths.push(mesh_path);
        }

        Ok(ConfigEnvironmentObject {
            texture_path,
            submesh_paths,
            joint_influences_indices: league_skeleton.modern_data.influences,
            joints: league_skeleton
                .modern_data
                .joints
                .iter()
                .map(|v| ConfigJoint {
                    name: v.name.clone(),
                    transform: Transform::from_matrix(v.local_transform),
                    inverse_bind_pose: v.inverse_bind_transform,
                    parent_index: v.parent_index,
                })
                .collect(),
            animation_graph: ConfigAnimationGraph { clip_paths },
        })
    }

    pub async fn save_environment_object_async(
        &self,
        skin: &str,
    ) -> Result<ConfigEnvironmentObject, LeagueLoaderError> {
        let (skin_character_data_properties, flat_map) = self.load_character_skin(&skin);

        let texture_path = skin_character_data_properties
            .skin_mesh_properties
            .texture
            .clone();
        let skeleton_path = skin_character_data_properties
            .skin_mesh_properties
            .skeleton
            .clone();

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

        let clip_paths = animation_graph_data
            .clip_data_map
            .iter()
            .filter_map(|(_, v)| match v {
                AnimationClipData::AtomicClipData {
                    animation_resource_data,
                } => Some(animation_resource_data.animation_file_path.clone()),
                AnimationClipData::Unknown => None,
            })
            .collect::<Vec<_>>();

        // 保存纹理和骨骼文件
        self.save_wad_entry_to_file_async(&texture_path).await?;
        self.save_wad_entry_to_file_async(&skeleton_path).await?;

        // 保存动画文件
        for clip_path in &clip_paths {
            self.save_wad_entry_to_file_async(clip_path).await?;
        }

        let mut submesh_paths = Vec::new();

        for (i, range) in league_skinned_mesh.ranges.iter().enumerate() {
            let mesh = skinned_mesh_to_intermediate(&league_skinned_mesh, i).unwrap();
            let mesh_path = format!("skin_meshes/{}.mesh", &range.name);

            let mut file = get_async_asset_writer(&mesh_path).await?;
            let mut buffer = Vec::new();
            mesh.write(&mut Cursor::new(&mut buffer))
                .map_err(|e| LeagueLoaderError::Io(io::Error::new(io::ErrorKind::Other, e)))?;
            file.write_all(&buffer).await?;
            file.flush().await?;

            submesh_paths.push(mesh_path);
        }

        Ok(ConfigEnvironmentObject {
            texture_path,
            submesh_paths,
            joint_influences_indices: league_skeleton.modern_data.influences,
            joints: league_skeleton
                .modern_data
                .joints
                .iter()
                .map(|v| ConfigJoint {
                    name: v.name.clone(),
                    transform: Transform::from_matrix(v.local_transform),
                    inverse_bind_pose: v.inverse_bind_transform,
                    parent_index: v.parent_index,
                })
                .collect(),
            animation_graph: ConfigAnimationGraph { clip_paths },
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
            .find(|v| v.ctype.hash == LeagueLoader::hash_bin("SkinCharacterDataProperties"))
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

    pub fn save_wad_entry_to_file(&self, path: &str) -> Result<(), LeagueLoaderError> {
        let mut reader = self.get_wad_entry_reader_by_path(path)?;
        let mut file = get_asset_writer(path).unwrap();
        io::copy(&mut reader, &mut file)?;
        Ok(())
    }

    pub async fn save_wad_entry_to_file_async(&self, path: &str) -> Result<(), LeagueLoaderError> {
        let buffer = self.get_wad_entry_buffer_by_path(path)?;
        let mut file = get_async_asset_writer(path).await?;
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

pub fn get_asset_writer(path: &str) -> Result<File, LeagueLoaderError> {
    let path = format!("assets/{}", path);
    println!("√ {}", path);
    ensure_dir_exists(&path)?;
    let file = File::create(path)?;
    Ok(file)
}

pub fn save_struct_to_file<T: Serialize>(path: &str, data: &T) -> Result<(), LeagueLoaderError> {
    let mut file = get_asset_writer(path)?;
    to_writer_pretty(&mut file, data, Default::default())?;
    Ok(())
}

// 异步版本的文件写入函数
pub async fn get_async_asset_writer(path: &str) -> Result<AsyncFile, LeagueLoaderError> {
    let path = format!("assets/{}", path);
    println!("√ {}", path);
    ensure_dir_exists(&path)?;
    let file = AsyncFile::create(path).await?;
    Ok(file)
}

pub async fn save_struct_to_file_async<T: Serialize>(
    path: &str,
    data: &T,
) -> Result<(), LeagueLoaderError> {
    let serialized = ron::ser::to_string_pretty(data, Default::default())?;
    let mut file = get_async_asset_writer(path).await?;
    file.write_all(serialized.as_bytes()).await?;
    file.flush().await?;
    Ok(())
}
