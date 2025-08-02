mod compressed;
mod skeleton;
mod uncompressed;

use crate::render::{BinQuat, BinVec3};
use binrw::binread;

pub use compressed::*;
pub use skeleton::*;
pub use uncompressed::*;

// ===================================================================================
// 1. 统一的动画数据结构 (Unified Animation Data)
// ===================================================================================

/// 统一的动画数据，无论是从压缩格式还是未压缩格式解析而来
#[derive(Debug, Clone)]
pub struct AnimationData {
    pub version: u32,
    pub flags: u32,
    pub joint_count: i32,
    pub frame_count: i32,
    pub duration: f32,
    pub fps: f32,
    pub joint_hashes: Vec<u32>,
    pub frames: Vec<UnifiedFrame>,
    pub vector_palette: Vec<BinVec3>,
    pub quat_palette: Vec<BinQuat>,
}

/// 统一的帧数据结构，适用于所有动画格式
#[derive(Debug, Clone, Copy)]
pub struct UnifiedFrame {
    /// 时间戳（对于未压缩格式，这是帧索引；对于压缩格式，这是实际时间）
    pub time: f32,
    /// 关节哈希或索引
    pub joint_hash: u32,
    /// 平移向量在调色板中的索引
    pub translation_id: u16,
    /// 缩放向量在调色板中的索引
    pub scale_id: u16,
    /// 旋转四元数在调色板中的索引
    pub rotation_id: u16,
}

// ===================================================================================
// 2. 动画文件解析器 (Animation File Parser)
// ===================================================================================

#[binread]
#[derive(Debug)]
#[br(little)]
pub enum AnimationFile {
    #[br(magic = b"r3d2canm")]
    Compressed(CompressedAnimationAsset),
    #[br(magic = b"r3d2anmd")]
    Uncompressed(UncompressedAnimationAsset),
}

impl AnimationFile {
    /// 将解析的动画文件转换为统一的动画数据结构
    pub fn into_unified(self) -> AnimationData {
        match self {
            AnimationFile::Compressed(compressed) => compressed.into(),
            AnimationFile::Uncompressed(uncompressed) => uncompressed.into(),
        }
    }
}

// ===================================================================================
// 3. 转换实现 (Conversion Implementations)
// ===================================================================================

impl From<CompressedAnimationAsset> for AnimationData {
    fn from(compressed: CompressedAnimationAsset) -> Self {
        let data = &compressed.data;

        // 转换压缩帧数据为统一格式
        let mut frames = Vec::new();
        let frame_duration = data.duration / data.frame_count as f32;

        for frame in &data.frames {
            let joint_hash = if frame.get_joint_id() < data.joint_hashes.len() as u16 {
                data.joint_hashes[frame.get_joint_id() as usize]
            } else {
                0 // 默认值，如果索引超出范围
            };

            frames.push(UnifiedFrame {
                time: frame.time as f32 * frame_duration,
                joint_hash,
                // 对于压缩格式，我们需要解压缩数据到调色板
                // 这里暂时使用占位符值，实际实现需要解压缩逻辑
                translation_id: frame.value[0],
                scale_id: frame.value[1],
                rotation_id: frame.value[2],
            });
        }

        // 为压缩格式创建基本的调色板
        let vector_palette = vec![
            BinVec3(data.translation_min.0),
            BinVec3(data.translation_max.0),
            BinVec3(data.scale_min.0),
            BinVec3(data.scale_max.0),
        ];
        let quat_palette = Vec::new();

        AnimationData {
            version: compressed.version,
            flags: data.flags,
            joint_count: data.joint_count,
            frame_count: data.frame_count,
            duration: data.duration,
            fps: data.fps,
            joint_hashes: data.joint_hashes.clone(),
            frames,
            vector_palette,
            quat_palette,
        }
    }
}

impl From<UncompressedAnimationAsset> for AnimationData {
    fn from(uncompressed: UncompressedAnimationAsset) -> Self {
        println!(
            "Converting uncompressed animation, version: {}",
            uncompressed.version
        );
        match &uncompressed.data {
            UncompressedData::V3(v3) => {
                // 转换V3格式的帧数据为统一格式
                let mut frames = Vec::new();
                let frame_duration = 1.0 / v3.fps as f32;

                for (&joint_hash, joint_frames) in &v3.joint_frames {
                    for (frame_idx, frame) in joint_frames.iter().enumerate() {
                        frames.push(UnifiedFrame {
                            time: frame_idx as f32 * frame_duration,
                            joint_hash,
                            translation_id: frame.translation_id,
                            scale_id: frame.scale_id,
                            rotation_id: frame.rotation_id,
                        });
                    }
                }

                AnimationData {
                    version: uncompressed.version,
                    flags: 0, // V3 没有 flags
                    joint_count: v3.track_count,
                    frame_count: v3.frame_count,
                    duration: v3.frame_count as f32 / v3.fps as f32,
                    fps: v3.fps as f32,
                    joint_hashes: v3.joint_frames.keys().cloned().collect(),
                    frames,
                    vector_palette: v3.vector_palette.clone(),
                    quat_palette: v3.quat_palette.clone(),
                }
            }
            UncompressedData::V4(v4) => {
                // 转换V4格式的帧数据为统一格式
                let mut frames = Vec::new();

                for (&joint_hash, joint_frames) in &v4.joint_frames {
                    for (frame_idx, frame) in joint_frames.iter().enumerate() {
                        frames.push(UnifiedFrame {
                            time: frame_idx as f32 * v4.frame_duration,
                            joint_hash,
                            translation_id: frame.translation_id,
                            scale_id: frame.scale_id,
                            rotation_id: frame.rotation_id,
                        });
                    }
                }

                AnimationData {
                    version: uncompressed.version,
                    flags: v4.flags,
                    joint_count: v4.track_count,
                    frame_count: v4.frame_count,
                    duration: v4.frame_duration * v4.frame_count as f32,
                    fps: 1.0 / v4.frame_duration,
                    joint_hashes: v4.joint_frames.keys().cloned().collect(),
                    frames,
                    vector_palette: v4.vector_palette.clone(),
                    quat_palette: v4.quat_palette.clone(),
                }
            }
            UncompressedData::V5(v5) => {
                // 转换V5格式的帧数据为统一格式
                let mut frames = Vec::new();
                let frames_per_joint = v5.frames.len() / v5.joint_hashes.len();

                // 调试输出
                println!(
                    "V5 Debug: frame_duration={}, frame_count={}, frames_per_joint={}",
                    v5.frame_duration, v5.frame_count, frames_per_joint
                );

                // 计算正确的帧时间间隔 - V5格式中frame_duration是每帧的时间
                let time_per_frame = v5.frame_duration;
                let total_duration = v5.frame_duration * frames_per_joint as f32;

                for (joint_idx, &joint_hash) in v5.joint_hashes.iter().enumerate() {
                    let start_idx = joint_idx * frames_per_joint;
                    for (frame_idx, frame) in v5.frames[start_idx..start_idx + frames_per_joint]
                        .iter()
                        .enumerate()
                    {
                        let frame_time = frame_idx as f32 * time_per_frame;
                        if joint_idx == 0 && frame_idx < 3 {
                            println!("Frame {}: time={}", frame_idx, frame_time);
                        }
                        frames.push(UnifiedFrame {
                            time: frame_time,
                            joint_hash,
                            translation_id: frame.translation_id,
                            scale_id: frame.scale_id,
                            rotation_id: frame.rotation_id,
                        });
                    }
                }

                AnimationData {
                    version: uncompressed.version,
                    flags: v5.flags,
                    joint_count: v5.track_count,
                    frame_count: v5.frame_count,
                    duration: total_duration,
                    fps: 1.0 / v5.frame_duration,
                    joint_hashes: v5.joint_hashes.clone(),
                    frames,
                    vector_palette: v5.vector_palette.clone(),
                    quat_palette: v5.quat_palette.clone(),
                }
            }
        }
    }
}
