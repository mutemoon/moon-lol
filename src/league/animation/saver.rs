use std::io::{self, Cursor};

use bevy::prelude::*;
use binrw::{BinRead, BinWrite};
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

use crate::core::{
    ConfigCharacterSkin, ConfigCharacterSkinAnimation, ConfigJoint,
    ConfigSkinnedMeshInverseBindposes,
};
use crate::league::{
    from_entry, get_asset_writer, get_bin_path, neg_mat_z, save_struct_to_file,
    skinned_mesh_to_intermediate, AnimationConditionUpdater, AnimationGraphData, ClipData,
    EntryData, LeagueLoader, LeagueLoaderError, LeagueMaterial, LeagueSkeleton, LeagueSkinnedMesh,
    LeagueSkinnedMeshInternal, LeagueWadLoader,
};

impl LeagueWadLoader {
    pub async fn save_environment_object(
        &self,
        skin: &str,
    ) -> Result<ConfigCharacterSkin, LeagueLoaderError> {
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

        let animation_map = self.load_animation_map(
            flat_map
                .get(
                    &skin_character_data_properties
                        .skin_animation_properties
                        .animation_graph_data,
                )
                .unwrap(),
        )?;

        // 保存动画文件
        for (_, animation) in &animation_map {
            match animation {
                ConfigCharacterSkinAnimation::AtomicClipData { clip_path, .. } => {
                    self.save_wad_entry_to_file(clip_path).await?;
                }
                _ => {}
            }
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
            .map(|v| neg_mat_z(&v))
            .collect::<Vec<_>>();

        let inverse_bind_pose_path = get_bin_path(&format!("ASSETS/{}/inverse_bind_pose", skin));
        save_struct_to_file(
            &inverse_bind_pose_path,
            &ConfigSkinnedMeshInverseBindposes {
                inverse_bindposes: inverse_bind_poses,
            },
        )
        .await?;

        Ok(ConfigCharacterSkin {
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
                    hash: LeagueLoader::hash_joint(&joint.name),
                    transform: Transform::from_matrix(neg_mat_z(&joint.local_transform)),
                    parent_index: joint.parent_index,
                })
                .collect(),
            animation_map,
        })
    }

    pub fn load_animation_map(
        &self,
        value: &EntryData,
    ) -> Result<HashMap<u32, ConfigCharacterSkinAnimation>, LeagueLoaderError> {
        let animation_graph_data = from_entry::<AnimationGraphData>(value);

        let nodes = animation_graph_data
            .m_clip_data_map
            .iter()
            .collect::<HashMap<_, _>>();

        let animation_graph_data = nodes
            .iter()
            .filter_map(|(&k, &v)| -> Option<(u32, ConfigCharacterSkinAnimation)> {
                match v {
                    ClipData::AtomicClipData(atomic_clip_data) => Some((
                        *k,
                        ConfigCharacterSkinAnimation::AtomicClipData {
                            clip_path: atomic_clip_data
                                .m_animation_resource_data
                                .m_animation_file_path
                                .clone(),
                        },
                    )),
                    ClipData::SelectorClipData(selector_clip_data) => Some((
                        *k,
                        ConfigCharacterSkinAnimation::SelectorClipData {
                            probably_nodes: selector_clip_data
                                .m_selector_pair_data_list
                                .iter()
                                .map(|v| (v.m_clip_name, v.m_probability))
                                .collect(),
                        },
                    )),
                    ClipData::ConditionFloatClipData(condition_float_clip_data) => Some((
                        *k,
                        ConfigCharacterSkinAnimation::ConditionFloatClipData {
                            conditions: condition_float_clip_data
                                .m_condition_float_pair_data_list
                                .iter()
                                .map(|v| (v.m_clip_name, v.m_value.unwrap_or(0.0)))
                                .collect(),
                            component_name: match condition_float_clip_data.updater {
                                AnimationConditionUpdater::MoveSpeedParametricUpdater => {
                                    "Movement".to_string()
                                }
                                _ => "".to_string(),
                            },
                            field_name: match condition_float_clip_data.updater {
                                AnimationConditionUpdater::MoveSpeedParametricUpdater => {
                                    "speed".to_string()
                                }
                                _ => "".to_string(),
                            },
                        },
                    )),
                    _ => None,
                }
            })
            .collect();

        Ok(animation_graph_data)
    }
}
