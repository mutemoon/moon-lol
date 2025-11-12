use std::collections::HashMap;

use bevy::prelude::*;
use binrw::BinRead;

use league_core::{
    AnimationGraphData, AnimationGraphDataMBlendDataTable, AnimationGraphDataMClipDataMap,
    AtomicClipData, VfxEmitterDefinitionDataPrimitive, VfxSystemDefinitionData,
};
use league_file::{
    AnimationFile, CompressedTransformType, LeagueSkeleton, LeagueSkinnedMesh, UncompressedData,
};
use league_loader::LeagueWadLoader;
use league_property::{from_entry, EntryData};
use league_utils::hash_joint;
use lol_config::{
    ConfigAnimationClip, ConfigCharacterSkin, ConfigJoint, ConfigSkinnedMeshInverseBindposes,
    LeagueMaterial,
};

use crate::{
    get_bin_path, save_struct_to_file, save_wad_entry_to_file, skinned_mesh_to_intermediate, Error,
};

pub async fn save_character(
    loader: &LeagueWadLoader,
    skin: &str,
) -> Result<HashMap<u32, VfxSystemDefinitionData>, Error> {
    let (skin_character_data_properties, resource_resolver, flat_map) =
        loader.load_character_skin(&skin);

    let mut vfx_system_definition_datas = HashMap::new();
    if let Some(Some(resource_map)) = resource_resolver.map(|v| v.resource_map) {
        for (hash, link) in resource_map {
            let Some(entry_data) = flat_map.get(&link) else {
                continue;
            };
            let vfx_system_definition_data = from_entry::<VfxSystemDefinitionData>(entry_data);

            if let Some(ref complex_emitter_definition_data) =
                vfx_system_definition_data.complex_emitter_definition_data
            {
                for v in complex_emitter_definition_data {
                    let Some(primitive) = &v.primitive else {
                        continue;
                    };

                    let VfxEmitterDefinitionDataPrimitive::VfxPrimitiveMesh(vfx_primitive_mesh) =
                        primitive
                    else {
                        continue;
                    };

                    let Some(m_mesh) = vfx_primitive_mesh.m_mesh.as_ref() else {
                        continue;
                    };

                    let Some(simple_mesh_name) = m_mesh.m_simple_mesh_name.as_ref() else {
                        continue;
                    };

                    save_wad_entry_to_file(loader, simple_mesh_name).await?;
                }
            };

            vfx_system_definition_datas.insert(hash, vfx_system_definition_data);
        }
    }

    let skin_mesh_properties = &skin_character_data_properties.skin_mesh_properties.unwrap();

    let texture = skin_mesh_properties.texture.clone().unwrap();
    save_wad_entry_to_file(loader, &texture).await?;

    let material = LeagueMaterial {
        texture_path: texture.clone(),
    };
    let material_path = get_bin_path(&format!("ASSETS/{}/material", skin));
    save_struct_to_file(&material_path, &material).await?;

    let skeleton = skin_mesh_properties.skeleton.clone().unwrap();
    save_wad_entry_to_file(loader, &skeleton).await?;

    let league_skeleton = loader
        .get_wad_entry_reader_by_path(&skeleton)
        .map(|mut v| LeagueSkeleton::read(&mut v).unwrap())
        .unwrap();

    let simple_skin = skin_mesh_properties.simple_skin.clone().unwrap();
    let mut reader = loader
        .get_wad_entry_no_seek_reader_by_path(&simple_skin)
        .unwrap();
    let league_simple_mesh = LeagueSkinnedMesh::read(&mut reader).unwrap();

    let (animation_map, blend_data) = load_animation_map(
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
            AnimationGraphDataMClipDataMap::AtomicClipData(AtomicClipData {
                m_animation_resource_data,
                ..
            }) => {
                let clip_path = &m_animation_resource_data.m_animation_file_path;
                let mut animation_file = loader.get_wad_entry_reader_by_path(&clip_path)?;
                let animation_file = AnimationFile::read(&mut animation_file)?;
                let animation_data = load_animation_file(animation_file);
                save_struct_to_file(&clip_path, &animation_data).await?;
            }
            _ => {}
        }
    }

    let mut submesh_paths = Vec::new();

    for (i, range) in league_simple_mesh.ranges.iter().enumerate() {
        let mesh = skinned_mesh_to_intermediate(&league_simple_mesh, i);
        let mesh_path = format!("ASSETS/{}/meshes/{}.mesh", skin, &range.name);
        save_struct_to_file(&mesh_path, &mesh).await?;

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

    let path = get_bin_path(&format!("ASSETS/{}/config_character_skin", skin));
    let config_character_skin = ConfigCharacterSkin {
        skin_scale: skin_mesh_properties.skin_scale,
        material_path,
        submesh_paths,
        joint_influences_indices: league_skeleton.modern_data.influences,
        inverse_bind_pose_path,
        joints: league_skeleton
            .modern_data
            .joints
            .iter()
            .map(|joint| ConfigJoint {
                hash: hash_joint(&joint.name),
                transform: Transform::from_matrix(joint.local_transform),
                parent_index: joint.parent_index,
            })
            .collect(),
        animation_map,
        blend_data,
    };
    save_struct_to_file(&path, &config_character_skin).await?;

    Ok(vfx_system_definition_datas)
}

pub fn load_animation_map(
    value: &EntryData,
) -> Result<
    (
        HashMap<u32, AnimationGraphDataMClipDataMap>,
        HashMap<(u32, u32), AnimationGraphDataMBlendDataTable>,
    ),
    Error,
> {
    let animation_graph_data = from_entry::<AnimationGraphData>(value);

    let (Some(nodes), Some(blend_data)) = (
        animation_graph_data.m_clip_data_map,
        animation_graph_data.m_blend_data_table,
    ) else {
        return Ok((HashMap::new(), HashMap::new()));
    };

    let blend_data = blend_data
        .into_iter()
        .map(|(k, v)| (((k >> 32) as u32, k as u32), v))
        .collect();

    Ok((nodes, blend_data))
}

pub fn load_animation_file(value: AnimationFile) -> ConfigAnimationClip {
    match value {
        AnimationFile::Compressed(compressed) => {
            let data = compressed.data;
            let joint_count = data.joint_count as usize;

            let mut translates = vec![Vec::new(); joint_count];
            let mut rotations = vec![Vec::new(); joint_count];
            let mut scales = vec![Vec::new(); joint_count];

            for frame in data.frames {
                let joint_id = frame.joint_id as usize;

                if joint_id >= joint_count {
                    panic!(
                        "索引 {} 超出关节索引范围 [0, {}]",
                        joint_id,
                        joint_count - 1
                    )
                }

                match frame.transform_type {
                    CompressedTransformType::Rotation => {
                        rotations[joint_id].push((frame.time, frame.rotation));
                    }
                    CompressedTransformType::Translation => {
                        translates[joint_id].push((frame.time, frame.translation));
                    }
                    CompressedTransformType::Scale => {
                        scales[joint_id].push((frame.time, frame.scale));
                    }
                }
            }

            ConfigAnimationClip {
                fps: data.fps,
                duration: data.duration,
                joint_hashes: data.joint_hashes,
                translates,
                rotations,
                scales,
            }
        }
        AnimationFile::Uncompressed(uncompressed) => match uncompressed.data {
            UncompressedData::V3(data) => {
                let fps = data.fps as f32;
                let duration = if fps > 0.0 {
                    (data.frame_count.saturating_sub(1)) as f32 / fps
                } else {
                    0.0
                };

                let mut joint_hashes: Vec<u32> = data.joint_frames.keys().cloned().collect();
                joint_hashes.sort_unstable();

                let hash_to_idx: HashMap<u32, usize> = joint_hashes
                    .iter()
                    .enumerate()
                    .map(|(i, &h)| (h, i))
                    .collect();

                let num_joints = joint_hashes.len();
                let mut translates = vec![Vec::new(); num_joints];
                let mut rotations = vec![Vec::new(); num_joints];
                let mut scales = vec![Vec::new(); num_joints];

                for (hash, frames) in data.joint_frames {
                    if let Some(&joint_idx) = hash_to_idx.get(&hash) {
                        for (time_idx, frame) in frames.iter().enumerate() {
                            let time = time_idx as f32 / fps;

                            rotations[joint_idx]
                                .push((time, data.quat_palette[frame.rotation_id as usize]));
                            translates[joint_idx]
                                .push((time, data.vector_palette[frame.translation_id as usize]));
                            scales[joint_idx]
                                .push((time, data.vector_palette[frame.scale_id as usize]));
                        }
                    }
                }

                ConfigAnimationClip {
                    fps,
                    duration,
                    joint_hashes,
                    translates,
                    rotations,
                    scales,
                }
            }
            UncompressedData::V4(data) => {
                let fps = if data.frame_duration > 0.0 {
                    1.0 / data.frame_duration
                } else {
                    0.0
                };
                let duration = data.frame_duration * (data.frame_count.saturating_sub(1)) as f32;

                let mut joint_hashes: Vec<u32> = data.joint_frames.keys().cloned().collect();
                joint_hashes.sort_unstable();

                let hash_to_idx: HashMap<u32, usize> = joint_hashes
                    .iter()
                    .enumerate()
                    .map(|(i, &h)| (h, i))
                    .collect();

                let num_joints = joint_hashes.len();
                let mut translates = vec![Vec::new(); num_joints];
                let mut rotations = vec![Vec::new(); num_joints];
                let mut scales = vec![Vec::new(); num_joints];

                for (hash, frames) in data.joint_frames {
                    if let Some(&joint_idx) = hash_to_idx.get(&hash) {
                        for (time_idx, frame) in frames.iter().enumerate() {
                            let time = time_idx as f32 * data.frame_duration;

                            rotations[joint_idx]
                                .push((time, data.quat_palette[frame.rotation_id as usize]));
                            translates[joint_idx]
                                .push((time, data.vector_palette[frame.translation_id as usize]));
                            scales[joint_idx]
                                .push((time, data.vector_palette[frame.scale_id as usize]));
                        }
                    }
                }

                ConfigAnimationClip {
                    fps,
                    duration,
                    joint_hashes,
                    translates,
                    rotations,
                    scales,
                }
            }
            UncompressedData::V5(data) => {
                let joint_count = data.track_count as usize;
                let frame_count = data.frame_count as usize;
                let fps = if data.frame_duration > 0.0 {
                    1.0 / data.frame_duration
                } else {
                    0.0
                };
                let duration = data.frame_duration * (frame_count.saturating_sub(1)) as f32;

                let joint_hashes = data.joint_hashes;
                assert_eq!(
                    joint_hashes.len(),
                    joint_count,
                    "V5中关节哈希数量与轨道数量不匹配"
                );

                let mut translates = vec![Vec::with_capacity(frame_count); joint_count];
                let mut rotations = vec![Vec::with_capacity(frame_count); joint_count];
                let mut scales = vec![Vec::with_capacity(frame_count); joint_count];

                for time_idx in 0..frame_count {
                    let time = time_idx as f32 * data.frame_duration;
                    for joint_idx in 0..joint_count {
                        let frame_idx = time_idx * joint_count + joint_idx;
                        if let Some(frame) = data.frames.get(frame_idx) {
                            if let Some(rot_quat) =
                                data.quat_palette.get(frame.rotation_id as usize)
                            {
                                rotations[joint_idx].push((time, *rot_quat));
                            }

                            if let Some(trans_bvec) =
                                data.vector_palette.get(frame.translation_id as usize)
                            {
                                translates[joint_idx].push((time, *trans_bvec));
                            }
                            if let Some(scale_bvec) =
                                data.vector_palette.get(frame.scale_id as usize)
                            {
                                scales[joint_idx].push((time, *scale_bvec));
                            }
                        }
                    }
                }

                ConfigAnimationClip {
                    fps,
                    duration,
                    joint_hashes,
                    translates,
                    rotations,
                    scales,
                }
            }
        },
    }
}
