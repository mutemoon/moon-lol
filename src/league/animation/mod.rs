mod compressed;
mod skeleton;
mod uncompressed;

use std::collections::HashMap;

pub use compressed::*;
pub use skeleton::*;
pub use uncompressed::*;

use bevy::math::{Quat, Vec3};
use binrw::binread;

#[binread]
#[derive(Debug)]
#[br(little)]
pub enum AnimationFile {
    #[br(magic = b"r3d2canm")]
    Compressed(CompressedAnimationAsset),
    #[br(magic = b"r3d2anmd")]
    Uncompressed(UncompressedAnimationAsset),
}

// 约束1：关节哈希数组的长度与三种关键帧数组的长度一致
// 约束2：关键帧的时间要小于动画时间
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

                // 初始化存储每个关节关键帧的向量
                let mut translates = vec![Vec::new(); joint_count];
                let mut rotations = vec![Vec::new(); joint_count];
                let mut scales = vec![Vec::new(); joint_count];

                // 遍历所有压缩帧
                for frame in data.frames {
                    let time = decompress_time(frame.time, duration);
                    let joint_id = frame.joint_id as usize;

                    // 确保 joint_id 不会越界
                    if joint_id >= joint_count {
                        panic!(
                            "索引 {} 超出关节索引范围 [0, {}]",
                            joint_id,
                            joint_count - 1
                        )
                    }

                    // 根据帧类型解压并存入相应的向量
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
            AnimationFile::Uncompressed(uncompressed) => {
                // 根据要求，未压缩动画的处理保留为 todo!()
                match uncompressed.data {
                    UncompressedData::V3(data) => {
                        let fps = data.fps as f32;
                        let duration = if fps > 0.0 {
                            (data.frame_count.saturating_sub(1)) as f32 / fps
                        } else {
                            0.0
                        };

                        // 为确保顺序确定性，对哈希进行排序
                        let mut joint_hashes: Vec<u32> =
                            data.joint_frames.keys().cloned().collect();
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

                                    // 假设 BinQuat 和 BinVec3 是元组结构，用 .0 访问
                                    rotations[joint_idx].push((
                                        time,
                                        data.quat_palette[frame.rotation_id as usize].0,
                                    ));
                                    translates[joint_idx].push((
                                        time,
                                        data.vector_palette[frame.translation_id as usize].0,
                                    ));
                                    scales[joint_idx].push((
                                        time,
                                        data.vector_palette[frame.scale_id as usize].0,
                                    ));
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

                        // 为确保顺序确定性，对哈希进行排序
                        let mut joint_hashes: Vec<u32> =
                            data.joint_frames.keys().cloned().collect();
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

                                    // 假设 BinQuat 和 BinVec3 是元组结构，用 .0 访问
                                    rotations[joint_idx].push((
                                        time,
                                        data.quat_palette[frame.rotation_id as usize].0,
                                    ));
                                    translates[joint_idx].push((
                                        time,
                                        data.vector_palette[frame.translation_id as usize].0,
                                    ));
                                    scales[joint_idx].push((
                                        time,
                                        data.vector_palette[frame.scale_id as usize].0,
                                    ));
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
                                    // V5的quat_palette已经是Vec<Quat>
                                    if let Some(rot_quat) =
                                        data.quat_palette.get(frame.rotation_id as usize)
                                    {
                                        rotations[joint_idx].push((time, *rot_quat));
                                    }
                                    // 假设 BinVec3 是元组结构，用 .0 访问
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
                }
            }
        }
    }
}
