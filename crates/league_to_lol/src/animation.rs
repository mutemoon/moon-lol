use std::collections::HashMap;

use bevy::prelude::*;

use league_core::{
    AnimationGraphData, AnimationGraphDataMBlendDataTable, AnimationGraphDataMClipDataMap,
};
use league_file::{AnimationFile, CompressedTransformType, UncompressedData};
use league_property::{from_entry_unwrap, EntryData};
use lol_config::ConfigAnimationClip;

use crate::Error;

pub fn load_animation_map(
    value: &EntryData,
) -> Result<
    (
        HashMap<u32, AnimationGraphDataMClipDataMap>,
        HashMap<(u32, u32), AnimationGraphDataMBlendDataTable>,
    ),
    Error,
> {
    let animation_graph_data = from_entry_unwrap::<AnimationGraphData>(value);

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
