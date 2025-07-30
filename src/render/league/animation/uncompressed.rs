use crate::render::{BinQuat, BinVec3, LeagueLoader};
use bevy::math::{Quat, Vec3};
use binrw::io::{Read, Seek, SeekFrom};
use binrw::{binread, BinRead};
use binrw::{prelude::*, Endian};
use std::collections::HashMap;
use std::f32::consts::SQRT_2;
use std::fmt::Debug;

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct UncompressedAnimationAsset {
    pub version: u32,

    #[br(args { version })]
    pub data: UncompressedData,
}

#[binread]
#[derive(Debug)]
#[br(little, import { version: u32 })]
pub enum UncompressedData {
    #[br(pre_assert(version == 3))]
    V3(#[br(parse_with = parse_uncompressed_data_v3)] UncompressedDataV3),
    #[br(pre_assert(version == 4))]
    V4(UncompressedDataV4),
    #[br(pre_assert(version == 5))]
    V5(UncompressedDataV5),
}

// ------------------- Version 5 -------------------
#[binread]
#[derive(Debug)]
#[br(little)]
pub struct UncompressedDataV5 {
    pub resource_size: u32,
    pub format_token: u32,
    pub version_again: u32, // Should be 5
    pub flags: u32,
    pub track_count: i32,
    pub frame_count: i32,
    pub frame_duration: f32,

    #[br(temp)]
    joint_name_hashes_offset: i32,
    #[br(temp)]
    asset_name_offset: i32,
    #[br(temp)]
    time_offset: i32,
    #[br(temp)]
    vector_palette_offset: i32,
    #[br(temp)]
    quat_palette_offset: i32,
    #[br(temp)]
    frames_offset: i32,

    #[br(
        seek_before = SeekFrom::Start(joint_name_hashes_offset as u64 + 12),
        count = (frames_offset - joint_name_hashes_offset) / 4
    )]
    pub joint_hashes: Vec<u32>,

    #[br(
        seek_before = SeekFrom::Start(vector_palette_offset as u64 + 12),
        count = (quat_palette_offset - vector_palette_offset) / 12
    )]
    pub vector_palette: Vec<BinVec3>,

    #[br(
        seek_before = SeekFrom::Start(quat_palette_offset as u64 + 12),
        count = (joint_name_hashes_offset - quat_palette_offset) / 6,
        map = |vals: Vec<[u8; 6]>| vals.into_iter().map(decompress_quat).map(|v| BinQuat(v)).collect()
    )]
    pub quat_palette: Vec<BinQuat>,

    #[br(
        seek_before = SeekFrom::Start(frames_offset as u64 + 12),
        count = track_count * frame_count
    )]
    pub frames: Vec<UncompressedFrame>,
}

// ------------------- Version 4 -------------------
#[binread]
#[derive(Debug)]
#[br(little)]
pub struct UncompressedDataV4 {
    pub resource_size: u32,
    pub format_token: u32,
    pub version_again: u32, // Should be 4
    pub flags: u32,
    pub track_count: i32,
    pub frame_count: i32,
    pub frame_duration: f32,

    #[br(temp)]
    joint_name_hashes_offset: i32,
    #[br(temp)]
    asset_name_offset: i32,
    #[br(temp)]
    time_offset: i32,
    #[br(temp)]
    vector_palette_offset: i32,
    #[br(temp)]
    quat_palette_offset: i32,
    #[br(temp)]
    frames_offset: i32,

    #[br(
        seek_before = SeekFrom::Start(vector_palette_offset as u64 + 12),
        count = (quat_palette_offset - vector_palette_offset) / 12
    )]
    pub vector_palette: Vec<BinVec3>,

    #[br(
        seek_before = SeekFrom::Start(quat_palette_offset as u64 + 12),
        count = (frames_offset - quat_palette_offset) / 16
    )]
    pub quat_palette: Vec<BinQuat>,

    #[br(
        seek_before = SeekFrom::Start(frames_offset as u64 + 12),
        count = (track_count * frame_count) as usize,
        map = |frames: Vec<UncompressedFrameV4>| group_v4_frames(frames)
    )]
    pub joint_frames: HashMap<u32, Vec<UncompressedFrame>>,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct UncompressedFrameV4 {
    pub joint_hash: u32,
    pub frame: UncompressedFrame,
    pub padding: u16,
}

fn group_v4_frames(frames: Vec<UncompressedFrameV4>) -> HashMap<u32, Vec<UncompressedFrame>> {
    let mut map = HashMap::new();
    for frame_v4 in frames {
        map.entry(frame_v4.joint_hash)
            .or_insert_with(Vec::new)
            .push(frame_v4.frame);
    }
    map
}

// ------------------- Version 3 (Legacy) -------------------

// 用于V3解析的辅助结构体。它们使用宏来读取数据块。
#[binread]
#[derive(Debug)]
#[br(little, import { frame_count: i32 })]
struct RawTrackV3 {
    track_name_bytes: [u8; 32],
    _flags: u32,
    #[br(count = frame_count)]
    frames: Vec<RawFrameV3>,
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct RawFrameV3 {
    rotation: BinQuat,
    translation: BinVec3,
}

/// V3 数据的最终结构。它本身不派生 BinRead。
#[derive(Debug)]
pub struct UncompressedDataV3 {
    pub skeleton_id: u32,
    pub track_count: i32,
    pub frame_count: i32,
    pub fps: i32,
    pub joint_frames: HashMap<u32, Vec<UncompressedFrame>>,
    pub vector_palette: Vec<BinVec3>,
    pub quat_palette: Vec<BinQuat>,
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct RawDataV3 {
    skeleton_id: u32,
    track_count: i32,
    frame_count: i32,
    fps: i32,
    #[br(count = track_count, args { inner: RawTrackV3BinReadArgs { frame_count } })]
    tracks: Vec<RawTrackV3>,
}

/// 自定义解析函数，它使用辅助结构体来读取原始数据，
/// 然后将其转换为最终的 UncompressedDataV3 结构。
/// 这取代了手动的 `impl BinRead`。
fn parse_uncompressed_data_v3<R: Read + Seek>(
    reader: &mut R,
    _: Endian,
    _: (),
) -> BinResult<UncompressedDataV3> {
    // 定义一个临时结构体，使用 binrw 宏来读取原始数据块

    // 从流中读取原始数据结构
    let raw = RawDataV3::read(reader)?;

    // 现在，将原始数据转换为所需的 UncompressedDataV3 结构
    let track_count = raw.track_count;
    let frame_count = raw.frame_count;
    let mut joint_frames = HashMap::with_capacity(track_count as usize);
    let palette_size = (track_count * frame_count) as usize;
    let mut quat_palette = Vec::with_capacity(palette_size);
    // +1 是为人为添加的静态缩放向量
    let mut vector_palette = Vec::with_capacity(palette_size + 1);
    vector_palette.push(BinVec3(Vec3::ONE));

    for (i, raw_track) in raw.tracks.into_iter().enumerate() {
        let track_name = String::from_utf8_lossy(&raw_track.track_name_bytes)
            .trim_end_matches('\0')
            .to_string();

        // C# 代码使用 Elf.HashLower，我们在此模拟一个简单的小写哈希
        // 为了完美匹配，你需要实现确切的 Elf 哈希算法。
        let joint_hash = LeagueLoader::compute_binhash(&track_name);

        let mut frames_for_joint = Vec::with_capacity(frame_count as usize);

        for (j, raw_frame) in raw_track.frames.into_iter().enumerate() {
            let index = i * frame_count as usize + j;

            quat_palette.push(raw_frame.rotation);
            vector_palette.push(raw_frame.translation);

            frames_for_joint.push(UncompressedFrame {
                rotation_id: index as u16,
                // 旧版格式可能没有缩放，C# 默认使用索引为 0 的静态 Vector3.One
                scale_id: 0,
                translation_id: (index + 1) as u16,
            });
        }
        joint_frames.insert(joint_hash, frames_for_joint);
    }

    Ok(UncompressedDataV3 {
        skeleton_id: raw.skeleton_id,
        track_count,
        frame_count,
        fps: raw.fps,
        joint_frames,
        vector_palette,
        quat_palette,
    })
}

// ------------------- 通用未压缩结构 -------------------
#[derive(BinRead, Debug, Clone, Copy)]
#[br(little)]
pub struct UncompressedFrame {
    pub translation_id: u16,
    pub scale_id: u16,
    pub rotation_id: u16,
}

// **************************************************************************
// * 辅助函数和类型
// **************************************************************************

// 修正后的函数
fn decompress_quat(data: [u8; 6]) -> Quat {
    // 1. 将6字节数据（3个u16）合并成一个u64，与C#逻辑保持一致
    // C#的ReadOnlySpan<ushort>和位移操作表明数据是按小端（Little Endian）u16处理的
    let d0 = u16::from_le_bytes([data[0], data[1]]);
    let d1 = u16::from_le_bytes([data[2], data[3]]);
    let d2 = u16::from_le_bytes([data[4], data[5]]);

    // 使用 u64 来进行位操作，避免溢出
    let bits: u64 = (d0 as u64) | ((d1 as u64) << 16) | ((d2 as u64) << 32);

    // 2. 严格按照 C# 的位布局提取数据
    let max_index = (bits >> 45) & 0x03; // 2-bit index
    let v_a = (bits >> 30) & 0x7FFF; // 15-bit value
    let v_b = (bits >> 15) & 0x7FFF; // 15-bit value
    let v_c = bits & 0x7FFF; // 15-bit value

    const ONE_OVER_SQRT_2: f32 = 0.70710678118; // 1.0 / SQRT_2
                                                // 3. 应用与 C# 完全相同的反量化公式
                                                // C# 中使用 double 进行除法和常量计算，这里用 f32 可能会有微小精度差异
    let a = (v_a as f32 / 32767.0) * SQRT_2 - ONE_OVER_SQRT_2;
    let b = (v_b as f32 / 32767.0) * SQRT_2 - ONE_OVER_SQRT_2;
    let c = (v_c as f32 / 32767.0) * SQRT_2 - ONE_OVER_SQRT_2;

    // 4. 计算第四个分量，并加入安全检查
    let sum_sq = a * a + b * b + c * c;
    let d = (1.0 - sum_sq).max(0.0).sqrt();

    // 5. 根据索引重构四元数
    // 这里的 Quat::from_xyzw 是一个示例，请使用你实际的库函数
    let quat = match max_index {
        0 => Quat::from_xyzw(d, a, b, c), // x was omitted
        1 => Quat::from_xyzw(a, d, b, c), // y was omitted
        2 => Quat::from_xyzw(a, b, d, c), // z was omitted
        _ => Quat::from_xyzw(a, b, c, d), // w was omitted
    };

    // C# 版本没有显式调用 normalize，因为计算结果理论上就是单位化的。
    // 如果需要更高的精度保证，可以取消下面这行注释。
    // return BinQuat(quat.normalize());

    quat
}
