use crate::render::BinVec3;
use binrw::io::{Read, Seek, SeekFrom};
use binrw::{binread, BinRead};
use binrw::{prelude::*, Endian};
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

// 自定义解析函数以处理不同版本的压缩数据
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
    pub flags: u32, // CompressedAnimationFlags
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

    // Offsets
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

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct CompressedFrame {
    pub time: u16,
    packed_data: u16,
    pub value: [u16; 3],
}

impl CompressedFrame {
    pub fn get_joint_id(&self) -> u16 {
        self.packed_data & 0x3FFF
    }

    pub fn get_transform_type(&self) -> u16 {
        self.packed_data >> 14
    }
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct ErrorMetric {
    pub average: f32,
    pub max: f32,
    pub total: f32,
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
