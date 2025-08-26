mod compressed;
mod loader;
mod saver;
mod skeleton;
mod uncompressed;

use std::collections::HashMap;

pub use compressed::*;
pub use loader::*;
pub use skeleton::*;
pub use uncompressed::*;

use bevy::prelude::AnimatedField;
use bevy::{
    animation::{
        animated_field,
        animation_curves::{AnimatableCurve, AnimatableKeyframeCurve},
        AnimationClip, AnimationTargetId,
    },
    asset::uuid::Uuid,
    math::{Quat, Vec3},
    transform::components::Transform,
};
use binrw::binread;

use crate::league::{neg_rotation_z, neg_vec_z};

#[binread]
#[derive(Debug)]
#[br(little)]
pub enum AnimationFile {
    #[br(magic = b"r3d2canm")]
    Compressed(CompressedAnimationAsset),
    #[br(magic = b"r3d2anmd")]
    Uncompressed(UncompressedAnimationAsset),
}

#[derive(Debug, Default)]
pub struct AnimationData {
    pub fps: f32,
    pub duration: f32,
    pub joint_hashes: Vec<u32>,
    pub translates: Vec<Vec<(f32, Vec3)>>,
    pub rotations: Vec<Vec<(f32, Quat)>>,
    pub scales: Vec<Vec<(f32, Vec3)>>,
}

impl From<AnimationFile> for AnimationData {
    fn from(value: AnimationFile) -> Self {
        match value {
            AnimationFile::Compressed(compressed) => {
                let data = compressed.data;
                let joint_count = data.joint_count as usize;
                let duration = data.duration;

                let mut translates = vec![Vec::new(); joint_count];
                let mut rotations = vec![Vec::new(); joint_count];
                let mut scales = vec![Vec::new(); joint_count];

                for frame in data.frames {
                    let time = decompress_time(frame.time, duration);
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
                            let quat = decompress_quat(&frame.value);
                            rotations[joint_id].push((time, quat));
                        }
                        CompressedTransformType::Translation => {
                            let vec = decompress_vector3(
                                &frame.value,
                                &data.translation_min,
                                &data.translation_max,
                            );
                            translates[joint_id].push((time, vec));
                        }
                        CompressedTransformType::Scale => {
                            let vec =
                                decompress_vector3(&frame.value, &data.scale_min, &data.scale_max);
                            scales[joint_id].push((time, vec));
                        }
                    }
                }

                AnimationData {
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
                                    .push((time, data.quat_palette[frame.rotation_id as usize].0));
                                translates[joint_idx].push((
                                    time,
                                    data.vector_palette[frame.translation_id as usize].0,
                                ));
                                scales[joint_idx]
                                    .push((time, data.vector_palette[frame.scale_id as usize].0));
                            }
                        }
                    }

                    AnimationData {
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
                    let duration =
                        data.frame_duration * (data.frame_count.saturating_sub(1)) as f32;

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
                                    .push((time, data.quat_palette[frame.rotation_id as usize].0));
                                translates[joint_idx].push((
                                    time,
                                    data.vector_palette[frame.translation_id as usize].0,
                                ));
                                scales[joint_idx]
                                    .push((time, data.vector_palette[frame.scale_id as usize].0));
                            }
                        }
                    }

                    AnimationData {
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
                                    translates[joint_idx].push((time, trans_bvec.0));
                                }
                                if let Some(scale_bvec) =
                                    data.vector_palette.get(frame.scale_id as usize)
                                {
                                    scales[joint_idx].push((time, scale_bvec.0));
                                }
                            }
                        }
                    }

                    AnimationData {
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
}

impl From<AnimationData> for AnimationClip {
    fn from(value: AnimationData) -> Self {
        let mut clip = AnimationClip::default();
        for (i, join_hash) in value.joint_hashes.iter().enumerate() {
            let translates = value.translates.get(i).unwrap();
            let rotations = value.rotations.get(i).unwrap();
            let scales = value.scales.get(i).unwrap();

            if translates.len() >= 2 {
                clip.add_curve_to_target(
                    AnimationTargetId(Uuid::from_u128(*join_hash as u128)),
                    AnimatableCurve::new(
                        animated_field!(Transform::translation),
                        AnimatableKeyframeCurve::new(
                            translates.iter().map(|(time, vec)| (*time, neg_vec_z(vec))),
                        )
                        .unwrap(),
                    ),
                );
            }

            if rotations.len() >= 2 {
                clip.add_curve_to_target(
                    AnimationTargetId(Uuid::from_u128(*join_hash as u128)),
                    AnimatableCurve::new(
                        animated_field!(Transform::rotation),
                        AnimatableKeyframeCurve::new(
                            rotations
                                .iter()
                                .map(|(time, quat)| (*time, neg_rotation_z(quat))),
                        )
                        .unwrap(),
                    ),
                );
            }

            if scales.len() >= 2 {
                clip.add_curve_to_target(
                    AnimationTargetId(Uuid::from_u128(*join_hash as u128)),
                    AnimatableCurve::new(
                        animated_field!(Transform::scale),
                        AnimatableKeyframeCurve::new(scales.clone().into_iter()).unwrap(),
                    ),
                );
            }
        }
        clip
    }
}

#[cfg(test)]
mod tests {

    use std::{io::Cursor, time::Instant};

    use binrw::BinRead;

    use crate::league::{AnimationFile, LeagueLoader};

    #[test]
    fn test_read() {
        let start = Instant::now();
        let loader =
            LeagueLoader::new(r"C:\Program Files (x86)\WeGameApps\英雄联盟\game", "bloom").unwrap();

        println!("{:?}", start.elapsed());

        for (_, entry) in loader.map_loader.wad_loader.wad.entries.clone() {
            let mut reader = loader
                .map_loader
                .wad_loader
                .get_wad_entry_reader(&entry)
                .unwrap();
            let mut buf = [0; 8];
            if reader.read_exact(&mut buf).is_err() {
                continue;
            }
            let Ok(text) = str::from_utf8(&buf) else {
                continue;
            };
            if text != "r3d2canm" && text != "r3d2anmd" {
                continue;
            }

            let mut reader = loader
                .map_loader
                .wad_loader
                .get_wad_entry_reader(&entry)
                .unwrap();
            let mut buf: Vec<u8> = Vec::new();
            reader.read_to_end(&mut buf).unwrap();
            let res = AnimationFile::read(&mut Cursor::new(buf));
            assert_eq!(res.is_ok(), true);
        }
    }
}
