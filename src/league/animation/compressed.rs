use crate::league::BinVec3;
use bevy::math::{Quat, Vec3};
use binrw::io::{Read, Seek, SeekFrom};
use binrw::{binread, BinRead};
use binrw::{prelude::*, Endian};
use std::f32::consts::SQRT_2;
use std::fmt::Debug;

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct CompressedAnimationAsset {
    pub version: u32,

    #[br(parse_with = parse_compressed_data)]
    #[br(args(version))]
    pub data: CompressedData,
}

fn parse_compressed_data<R: Read + Seek>(
    reader: &mut R,
    _: Endian,
    args: (u32,),
) -> BinResult<CompressedData> {
    let version = args.0;
    if !(1..=3).contains(&version) {
        return Err(binrw::Error::AssertFail {
            pos: reader.stream_position().unwrap_or(0),
            message: format!("无效的压缩动画版本: {}", version),
        });
    }
    CompressedData::read_le(reader)
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct CompressedData {
    pub resource_size: u32,
    pub format_token: u32,
    pub flags: u32,
    pub joint_count: i32,
    pub frame_count: i32,
    pub jump_cache_count: i32,
    pub duration: f32,
    pub fps: f32,
    pub rotation_error_metric: ErrorMetric,
    pub translation_error_metric: ErrorMetric,
    pub scale_error_metric: ErrorMetric,
    pub translation_min: BinVec3,
    pub translation_max: BinVec3,
    pub scale_min: BinVec3,
    pub scale_max: BinVec3,

    #[br(temp)]
    frames_offset: i32,
    #[br(temp)]
    jump_caches_offset: i32,
    #[br(temp)]
    joint_name_hashes_offset: i32,

    #[br(
        seek_before = SeekFrom::Start(joint_name_hashes_offset as u64 + 12),
        count = joint_count
    )]
    pub joint_hashes: Vec<u32>,

    #[br(
        seek_before = SeekFrom::Start(frames_offset as u64 + 12),
        count = frame_count
    )]
    pub frames: Vec<CompressedFrame>,

    #[br(
        seek_before = SeekFrom::Start(jump_caches_offset as u64 + 12),
        parse_with = parse_jump_caches,
        args(joint_count, frame_count, jump_cache_count)
    )]
    pub jump_caches: JumpCaches,
}

#[derive(Debug, PartialEq)]
pub enum CompressedTransformType {
    Translation,
    Rotation,
    Scale,
}

impl From<u16> for CompressedTransformType {
    fn from(value: u16) -> Self {
        match value {
            0 => CompressedTransformType::Rotation,
            1 => CompressedTransformType::Translation,
            2 => CompressedTransformType::Scale,
            _ => panic!("未知的动画数据类型"),
        }
    }
}

#[binread]
#[derive(Debug, PartialEq)]
#[br(little)]
pub struct CompressedFrame {
    pub time: u16,

    #[br(temp)]
    joint_id_and_type: u16,

    pub value: [u16; 3],

    #[br(calc = (joint_id_and_type >> 14).into())]
    pub transform_type: CompressedTransformType,

    #[br(calc = joint_id_and_type & 0x3FFF)]
    pub joint_id: u16,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct ErrorMetric {
    pub error_margin: f32,
    pub discontinuity_threshold: f32,
}

#[derive(Debug)]
pub enum JumpCaches {
    U16(Vec<JumpFrameU16>),
    U32(Vec<JumpFrameU32>),
}

#[derive(BinRead, Debug, Clone, Copy)]
#[br(little)]
pub struct JumpFrameU16 {
    pub rotation_keys: [u16; 4],
    pub translation_keys: [u16; 4],
    pub scale_keys: [u16; 4],
}

#[derive(BinRead, Debug, Clone, Copy)]
#[br(little)]
pub struct JumpFrameU32 {
    pub rotation_keys: [u32; 4],
    pub translation_keys: [u32; 4],
    pub scale_keys: [u32; 4],
}

fn parse_jump_caches<R: Read + Seek>(
    reader: &mut R,
    options: Endian,
    args: (i32, i32, i32),
) -> BinResult<JumpCaches> {
    let (joint_count, frame_count, jump_cache_count) = args;
    let total_entries = (joint_count * jump_cache_count) as usize;

    if frame_count < 0x10001 {
        let mut frames = Vec::with_capacity(total_entries);
        for _ in 0..total_entries {
            frames.push(JumpFrameU16::read_options(reader, options, ())?);
        }
        Ok(JumpCaches::U16(frames))
    } else {
        let mut frames = Vec::with_capacity(total_entries);
        for _ in 0..total_entries {
            frames.push(JumpFrameU32::read_options(reader, options, ())?);
        }
        Ok(JumpCaches::U32(frames))
    }
}

const ONE_OVER_USHORT_MAX: f32 = 1.0 / 65535.0;
const ONE_OVER_SQRT_2: f32 = 1.0 / SQRT_2;

pub fn decompress_time(time: u16, duration: f32) -> f32 {
    time as f32 * ONE_OVER_USHORT_MAX * duration
}

pub fn decompress_vector3(value: &[u16; 3], min: &BinVec3, max: &BinVec3) -> Vec3 {
    let min_vec = Vec3::new(min.0.x, min.0.y, min.0.z);
    let max_vec = Vec3::new(max.0.x, max.0.y, max.0.z);

    let mut uncompressed = max_vec - min_vec;

    uncompressed.x *= value[0] as f32 * ONE_OVER_USHORT_MAX;
    uncompressed.y *= value[1] as f32 * ONE_OVER_USHORT_MAX;
    uncompressed.z *= value[2] as f32 * ONE_OVER_USHORT_MAX;

    uncompressed + min_vec
}

pub fn decompress_quat(value: &[u16; 3]) -> Quat {
    let bits = (value[0] as u64) | ((value[1] as u64) << 16) | ((value[2] as u64) << 32);

    let max_index = (bits >> 45) & 0x03;
    let v_a = (bits >> 30) & 0x7FFF;
    let v_b = (bits >> 15) & 0x7FFF;
    let v_c = bits & 0x7FFF;

    let a = (v_a as f32 / 32767.0) * SQRT_2 - ONE_OVER_SQRT_2;
    let b = (v_b as f32 / 32767.0) * SQRT_2 - ONE_OVER_SQRT_2;
    let c = (v_c as f32 / 32767.0) * SQRT_2 - ONE_OVER_SQRT_2;

    let sub = 1.0 - (a * a + b * b + c * c);
    let d = f32::sqrt(f32::max(0.0, sub));

    match max_index {
        0 => Quat::from_xyzw(d, a, b, c),
        1 => Quat::from_xyzw(a, d, b, c),
        2 => Quat::from_xyzw(a, b, d, c),
        _ => Quat::from_xyzw(a, b, c, d),
    }
}
