mod compressed;
mod skeleton;
mod uncompressed;

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
            AnimationFile::Uncompressed(_uncompressed_animation_asset) => {
                // 根据要求，未压缩动画的处理保留为 todo!()
                todo!("Uncompressed animation parsing not implemented")
            }
        }
    }
}
