use std::collections::HashMap;
use std::fmt::Debug;

use bevy::math::Quat;
use bevy::math::Vec3;
use binrw::io::{Read, Seek, SeekFrom};
use binrw::{binread, BinRead};
use binrw::{prelude::*, Endian};

use league_utils::animation::decompress_quat;
use league_utils::{hash_joint, parse_quat, parse_quat_array, parse_vec3, parse_vec3_array};

#[binread]
#[derive(Debug)]
#[br(little)]
pub enum AnimationFile {
    #[br(magic = b"r3d2canm")]
    Compressed(CompressedAnimationAsset),
    #[br(magic = b"r3d2anmd")]
    Uncompressed(UncompressedAnimationAsset),
}

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
    #[br(map = parse_vec3)]
    pub translation_min: Vec3,
    #[br(map = parse_vec3)]
    pub translation_max: Vec3,
    #[br(map = parse_vec3)]
    pub scale_min: Vec3,
    #[br(map = parse_vec3)]
    pub scale_max: Vec3,

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

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct UncompressedAnimationAsset {
    pub version: u32,

    #[br(args { version })]
    pub data: UncompressedData,
}

#[binread]
#[derive(Debug, Clone)]
#[br(little, import { version: u32 })]
pub enum UncompressedData {
    #[br(pre_assert(version == 3))]
    V3(#[br(parse_with = parse_uncompressed_data_v3)] UncompressedDataV3),
    #[br(pre_assert(version == 4))]
    V4(UncompressedDataV4),
    #[br(pre_assert(version == 5))]
    V5(UncompressedDataV5),
}

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct UncompressedDataV5 {
    pub resource_size: u32,
    pub format_token: u32,
    pub version_again: u32,
    pub flags: u32,
    pub track_count: i32,
    pub frame_count: i32,
    pub frame_duration: f32,

    #[br(temp)]
    joint_name_hashes_offset: i32,
    #[br(temp)]
    _asset_name_offset: i32,
    #[br(temp)]
    _time_offset: i32,
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
        count = (quat_palette_offset - vector_palette_offset) / 12,
        map = parse_vec3_array
    )]
    pub vector_palette: Vec<Vec3>,

    #[br(
        seek_before = SeekFrom::Start(quat_palette_offset as u64 + 12),
        count = (joint_name_hashes_offset - quat_palette_offset) / 6,
        map = |vals: Vec<[u16; 3]>| vals.iter().map(decompress_quat).collect()
    )]
    pub quat_palette: Vec<Quat>,

    #[br(
        seek_before = SeekFrom::Start(frames_offset as u64 + 12),
        count = track_count * frame_count
    )]
    pub frames: Vec<UncompressedFrame>,
}

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct UncompressedDataV4 {
    pub resource_size: u32,
    pub format_token: u32,
    pub version_again: u32,
    pub flags: u32,
    pub track_count: i32,
    pub frame_count: i32,
    pub frame_duration: f32,

    #[br(temp)]
    _joint_name_hashes_offset: i32,
    #[br(temp)]
    _asset_name_offset: i32,
    #[br(temp)]
    _time_offset: i32,
    #[br(temp)]
    vector_palette_offset: i32,
    #[br(temp)]
    quat_palette_offset: i32,
    #[br(temp)]
    frames_offset: i32,

    #[br(
        seek_before = SeekFrom::Start(vector_palette_offset as u64 + 12),
        count = (quat_palette_offset - vector_palette_offset) / 12,
        map = parse_vec3_array
    )]
    pub vector_palette: Vec<Vec3>,

    #[br(
        seek_before = SeekFrom::Start(quat_palette_offset as u64 + 12),
        count = (frames_offset - quat_palette_offset) / 16,
        map = parse_quat_array
    )]
    pub quat_palette: Vec<Quat>,

    #[br(
        seek_before = SeekFrom::Start(frames_offset as u64 + 12),
        count = (track_count * frame_count) as usize,
        map = |frames: Vec<UncompressedFrameV4>| group_v4_frames(frames)
    )]
    pub joint_frames: HashMap<u32, Vec<UncompressedFrame>>,
}

#[binread]
#[derive(Debug, Clone)]
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

#[binread]
#[derive(Debug, Clone)]
#[br(little, import { frame_count: i32 })]
struct RawTrackV3 {
    track_name_bytes: [u8; 32],
    _flags: u32,
    #[br(count = frame_count)]
    frames: Vec<RawFrameV3>,
}

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
struct RawFrameV3 {
    #[br(map = parse_quat)]
    rotation: Quat,
    #[br(map = parse_vec3)]
    translation: Vec3,
}

#[derive(Debug, Clone)]
pub struct UncompressedDataV3 {
    pub skeleton_id: u32,
    pub track_count: i32,
    pub frame_count: i32,
    pub fps: i32,
    pub joint_frames: HashMap<u32, Vec<UncompressedFrame>>,
    pub vector_palette: Vec<Vec3>,
    pub quat_palette: Vec<Quat>,
}

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
struct RawDataV3 {
    skeleton_id: u32,
    track_count: i32,
    frame_count: i32,
    fps: i32,
    #[br(count = track_count, args { inner: RawTrackV3BinReadArgs { frame_count } })]
    tracks: Vec<RawTrackV3>,
}

fn parse_uncompressed_data_v3<R: Read + Seek>(
    reader: &mut R,
    _: Endian,
    _: (),
) -> BinResult<UncompressedDataV3> {
    let raw = RawDataV3::read(reader)?;

    let track_count = raw.track_count;
    let frame_count = raw.frame_count;
    let mut joint_frames = HashMap::with_capacity(track_count as usize);
    let palette_size = (track_count * frame_count) as usize;
    let mut quat_palette = Vec::with_capacity(palette_size);

    let mut vector_palette = Vec::with_capacity(palette_size + 1);
    vector_palette.push(Vec3::ONE);

    for (i, raw_track) in raw.tracks.into_iter().enumerate() {
        let track_name = String::from_utf8_lossy(&raw_track.track_name_bytes)
            .trim_end_matches('\0')
            .to_string();

        let joint_hash = hash_joint(&track_name);

        let mut frames_for_joint = Vec::with_capacity(frame_count as usize);

        for (j, raw_frame) in raw_track.frames.into_iter().enumerate() {
            let index = i * frame_count as usize + j;

            quat_palette.push(raw_frame.rotation);
            vector_palette.push(raw_frame.translation);

            frames_for_joint.push(UncompressedFrame {
                rotation_id: index as u16,

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

#[derive(BinRead, Debug, Clone, Copy)]
#[br(little)]
pub struct UncompressedFrame {
    pub translation_id: u16,
    pub scale_id: u16,
    pub rotation_id: u16,
}
