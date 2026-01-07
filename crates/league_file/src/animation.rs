use std::collections::HashMap;
use std::f32::consts::SQRT_2;
use std::fmt::Debug;

use bevy::math::{Quat, Vec3};
use nom::bytes::complete::take;
use nom::multi::count;
use nom::number::complete::{le_f32, le_i32, le_u16, le_u32};
use nom::{IResult, Parser};

#[derive(Debug)]
pub enum AnimationFile {
    Compressed(CompressedAnimationAsset),
    Uncompressed(UncompressedAnimationAsset),
}

impl AnimationFile {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, magic) = take(8usize)(input)?;
        let (i, version) = le_u32(i)?;
        match magic {
            b"r3d2canm" => {
                let (_, data) = CompressedData::parse(input, version)?;
                Ok((
                    i,
                    AnimationFile::Compressed(CompressedAnimationAsset { version, data }),
                ))
            }
            b"r3d2anmd" => {
                let (_, data) = UncompressedData::parse(input, version)?;
                Ok((
                    i,
                    AnimationFile::Uncompressed(UncompressedAnimationAsset { version, data }),
                ))
            }
            _ => Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            ))),
        }
    }
}

#[derive(Debug)]
pub struct CompressedAnimationAsset {
    pub version: u32,
    pub data: CompressedData,
}

#[derive(Debug)]
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
    pub translation_min: Vec3,
    pub translation_max: Vec3,
    pub scale_min: Vec3,
    pub scale_max: Vec3,
    pub joint_hashes: Vec<u32>,
    pub frames: Vec<CompressedFrame>,
    pub jump_caches: JumpCaches,
}

impl CompressedData {
    pub fn parse(full_input: &[u8], _version: u32) -> IResult<&[u8], Self> {
        let i = &full_input[12..];
        let (i, resource_size) = le_u32(i)?;
        let (i, format_token) = le_u32(i)?;
        let (i, flags) = le_u32(i)?;
        let (i, joint_count) = le_i32(i)?;
        let (i, frame_count) = le_i32(i)?;
        let (i, jump_cache_count) = le_i32(i)?;
        let (i, duration) = le_f32(i)?;
        let (i, fps) = le_f32(i)?;

        let (i, rotation_error_metric) = ErrorMetric::parse(i)?;
        let (i, translation_error_metric) = ErrorMetric::parse(i)?;
        let (i, scale_error_metric) = ErrorMetric::parse(i)?;

        let (i, translation_min) = parse_vec3(i)?;
        let (i, translation_max) = parse_vec3(i)?;
        let (i, scale_min) = parse_vec3(i)?;
        let (i, scale_max) = parse_vec3(i)?;

        let (i, frames_offset) = le_i32(i)?;
        let (i, jump_caches_offset) = le_i32(i)?;
        let (i, joint_name_hashes_offset) = le_i32(i)?;

        let joint_hashes_start = joint_name_hashes_offset as usize + 12;
        let (_, joint_hashes) =
            count(le_u32, joint_count as usize).parse(&full_input[joint_hashes_start..])?;

        let frames_start = frames_offset as usize + 12;
        let (_, frames) = count(
            |input| {
                CompressedFrame::parse(
                    input,
                    duration,
                    translation_min,
                    translation_max,
                    scale_min,
                    scale_max,
                )
            },
            frame_count as usize,
        )
        .parse(&full_input[frames_start..])?;

        let jump_caches_start = jump_caches_offset as usize + 12;
        let (_, jump_caches) = JumpCaches::parse(
            &full_input[jump_caches_start..],
            joint_count,
            frame_count,
            jump_cache_count,
        )?;

        Ok((
            i,
            CompressedData {
                resource_size,
                format_token,
                flags,
                joint_count,
                frame_count,
                jump_cache_count,
                duration,
                fps,
                rotation_error_metric,
                translation_error_metric,
                scale_error_metric,
                translation_min,
                translation_max,
                scale_min,
                scale_max,
                joint_hashes,
                frames,
                jump_caches,
            },
        ))
    }
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

#[derive(Debug, PartialEq)]
pub struct CompressedFrame {
    pub time: f32,
    pub transform_type: CompressedTransformType,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub joint_id: u16,
}

impl CompressedFrame {
    pub fn parse<'a>(
        input: &'a [u8],
        duration: f32,
        translation_min: Vec3,
        translation_max: Vec3,
        scale_min: Vec3,
        scale_max: Vec3,
    ) -> IResult<&'a [u8], Self> {
        let (i, time_raw) = le_u16(input)?;
        let (i, joint_id_and_type) = le_u16(i)?;
        let (i, v0) = le_u16(i)?;
        let (i, v1) = le_u16(i)?;
        let (i, v2) = le_u16(i)?;

        let time = decompress_time(time_raw, duration);
        let transform_type = CompressedTransformType::from(joint_id_and_type >> 14);
        let joint_id = joint_id_and_type & 0x3FFF;
        let value = [v0, v1, v2];

        let mut translation = Vec3::ZERO;
        let mut rotation = Quat::IDENTITY;
        let mut scale = Vec3::ONE;

        match transform_type {
            CompressedTransformType::Translation => {
                translation = decompress_vector3(&value, &translation_min, &translation_max);
            }
            CompressedTransformType::Rotation => {
                rotation = decompress_quat(value);
            }
            CompressedTransformType::Scale => {
                scale = decompress_vector3(&value, &scale_min, &scale_max);
            }
        }

        Ok((
            i,
            CompressedFrame {
                time,
                transform_type,
                translation,
                rotation,
                scale,
                joint_id,
            },
        ))
    }
}

#[derive(Debug)]
pub struct ErrorMetric {
    pub error_margin: f32,
    pub discontinuity_threshold: f32,
}

impl ErrorMetric {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, error_margin) = le_f32(input)?;
        let (i, discontinuity_threshold) = le_f32(i)?;
        Ok((
            i,
            ErrorMetric {
                error_margin,
                discontinuity_threshold,
            },
        ))
    }
}

#[derive(Debug)]
pub enum JumpCaches {
    U16(Vec<JumpFrameU16>),
    U32(Vec<JumpFrameU32>),
}

impl JumpCaches {
    pub fn parse(
        input: &[u8],
        joint_count: i32,
        frame_count: i32,
        jump_cache_count: i32,
    ) -> IResult<&[u8], Self> {
        let total_entries = (joint_count * jump_cache_count) as usize;
        if frame_count < 0x10001 {
            let (i, frames) = count(JumpFrameU16::parse, total_entries).parse(input)?;
            Ok((i, JumpCaches::U16(frames)))
        } else {
            let (i, frames) = count(JumpFrameU32::parse, total_entries).parse(input)?;
            Ok((i, JumpCaches::U32(frames)))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JumpFrameU16 {
    pub rotation_keys: [u16; 4],
    pub translation_keys: [u16; 4],
    pub scale_keys: [u16; 4],
}

impl JumpFrameU16 {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, rotation_keys) = count(le_u16, 4).parse(input)?;
        let (i, translation_keys) = count(le_u16, 4).parse(i)?;
        let (i, scale_keys) = count(le_u16, 4).parse(i)?;
        Ok((
            i,
            JumpFrameU16 {
                rotation_keys: rotation_keys.try_into().unwrap(),
                translation_keys: translation_keys.try_into().unwrap(),
                scale_keys: scale_keys.try_into().unwrap(),
            },
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JumpFrameU32 {
    pub rotation_keys: [u32; 4],
    pub translation_keys: [u32; 4],
    pub scale_keys: [u32; 4],
}

impl JumpFrameU32 {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, rotation_keys) = count(le_u32, 4).parse(input)?;
        let (i, translation_keys) = count(le_u32, 4).parse(i)?;
        let (i, scale_keys) = count(le_u32, 4).parse(i)?;
        Ok((
            i,
            JumpFrameU32 {
                rotation_keys: rotation_keys.try_into().unwrap(),
                translation_keys: translation_keys.try_into().unwrap(),
                scale_keys: scale_keys.try_into().unwrap(),
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub struct UncompressedAnimationAsset {
    pub version: u32,
    pub data: UncompressedData,
}

#[derive(Debug, Clone)]
pub enum UncompressedData {
    V3(UncompressedDataV3),
    V4(UncompressedDataV4),
    V5(UncompressedDataV5),
}

impl UncompressedData {
    pub fn parse(full_input: &[u8], version: u32) -> IResult<&[u8], Self> {
        match version {
            3 => UncompressedDataV3::parse(full_input).map(|(i, v)| (i, UncompressedData::V3(v))),
            4 => UncompressedDataV4::parse(full_input).map(|(i, v)| (i, UncompressedData::V4(v))),
            5 => UncompressedDataV5::parse(full_input).map(|(i, v)| (i, UncompressedData::V5(v))),
            _ => Err(nom::Err::Error(nom::error::Error::new(
                full_input,
                nom::error::ErrorKind::Tag,
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UncompressedDataV5 {
    pub resource_size: u32,
    pub format_token: u32,
    pub version_again: u32,
    pub flags: u32,
    pub track_count: i32,
    pub frame_count: i32,
    pub frame_duration: f32,
    pub joint_hashes: Vec<u32>,
    pub vector_palette: Vec<Vec3>,
    pub quat_palette: Vec<Quat>,
    pub frames: Vec<UncompressedFrame>,
}

impl UncompressedDataV5 {
    pub fn parse(full_input: &[u8]) -> IResult<&[u8], Self> {
        let i = &full_input[12..];
        let (i, resource_size) = le_u32(i)?;
        let (i, format_token) = le_u32(i)?;
        let (i, version_again) = le_u32(i)?;
        let (i, flags) = le_u32(i)?;
        let (i, track_count) = le_i32(i)?;
        let (i, frame_count) = le_i32(i)?;
        let (i, frame_duration) = le_f32(i)?;

        let (i, joint_name_hashes_offset) = le_i32(i)?;
        let (i, _asset_name_offset) = le_i32(i)?;
        let (i, _time_offset) = le_i32(i)?;
        let (i, vector_palette_offset) = le_i32(i)?;
        let (i, quat_palette_offset) = le_i32(i)?;
        let (i, frames_offset) = le_i32(i)?;

        let joint_hashes_start = joint_name_hashes_offset as usize + 12;
        let joint_hashes_count = (frames_offset - joint_name_hashes_offset) / 4;
        let (_, joint_hashes) =
            count(le_u32, joint_hashes_count as usize).parse(&full_input[joint_hashes_start..])?;

        let vector_palette_start = vector_palette_offset as usize + 12;
        let vector_palette_count = (quat_palette_offset - vector_palette_offset) / 12;
        let (_, vector_palette) = count(parse_vec3, vector_palette_count as usize)
            .parse(&full_input[vector_palette_start..])?;

        let quat_palette_start = quat_palette_offset as usize + 12;
        let quat_palette_count = (joint_name_hashes_offset - quat_palette_offset) / 6;
        let (_, quat_palette) = count(
            |input| {
                let (input, v0) = le_u16(input)?;
                let (input, v1) = le_u16(input)?;
                let (input, v2) = le_u16(input)?;
                Ok((input, decompress_quat([v0, v1, v2])))
            },
            quat_palette_count as usize,
        )
        .parse(&full_input[quat_palette_start..])?;

        let frames_start = frames_offset as usize + 12;
        let frames_count = (track_count * frame_count) as usize;
        let (_, frames) =
            count(UncompressedFrame::parse, frames_count).parse(&full_input[frames_start..])?;

        Ok((
            i,
            UncompressedDataV5 {
                resource_size,
                format_token,
                version_again,
                flags,
                track_count,
                frame_count,
                frame_duration,
                joint_hashes,
                vector_palette,
                quat_palette,
                frames,
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub struct UncompressedDataV4 {
    pub resource_size: u32,
    pub format_token: u32,
    pub version_again: u32,
    pub flags: u32,
    pub track_count: i32,
    pub frame_count: i32,
    pub frame_duration: f32,
    pub vector_palette: Vec<Vec3>,
    pub quat_palette: Vec<Quat>,
    pub joint_frames: HashMap<u32, Vec<UncompressedFrame>>,
}

impl UncompressedDataV4 {
    pub fn parse(full_input: &[u8]) -> IResult<&[u8], Self> {
        let i = &full_input[12..];
        let (i, resource_size) = le_u32(i)?;
        let (i, format_token) = le_u32(i)?;
        let (i, version_again) = le_u32(i)?;
        let (i, flags) = le_u32(i)?;
        let (i, track_count) = le_i32(i)?;
        let (i, frame_count) = le_i32(i)?;
        let (i, frame_duration) = le_f32(i)?;

        let (i, _joint_name_hashes_offset) = le_i32(i)?;
        let (i, _asset_name_offset) = le_i32(i)?;
        let (i, _time_offset) = le_i32(i)?;
        let (i, vector_palette_offset) = le_i32(i)?;
        let (i, quat_palette_offset) = le_i32(i)?;
        let (i, frames_offset) = le_i32(i)?;

        let vector_palette_start = vector_palette_offset as usize + 12;
        let vector_palette_count = (quat_palette_offset - vector_palette_offset) / 12;
        let (_, vector_palette) = count(parse_vec3, vector_palette_count as usize)
            .parse(&full_input[vector_palette_start..])?;

        let quat_palette_start = quat_palette_offset as usize + 12;
        let quat_palette_count = (frames_offset - quat_palette_offset) / 16;
        let (_, quat_palette) = count(parse_quat, quat_palette_count as usize)
            .parse(&full_input[quat_palette_start..])?;

        let frames_start = frames_offset as usize + 12;
        let frames_count = (track_count * frame_count) as usize;
        let (_, frames_v4) =
            count(UncompressedFrameV4::parse, frames_count).parse(&full_input[frames_start..])?;

        Ok((
            i,
            UncompressedDataV4 {
                resource_size,
                format_token,
                version_again,
                flags,
                track_count,
                frame_count,
                frame_duration,
                vector_palette,
                quat_palette,
                joint_frames: group_v4_frames(frames_v4),
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub struct UncompressedFrameV4 {
    pub joint_hash: u32,
    pub frame: UncompressedFrame,
    pub padding: u16,
}

impl UncompressedFrameV4 {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, joint_hash) = le_u32(input)?;
        let (i, frame) = UncompressedFrame::parse(i)?;
        let (i, padding) = le_u16(i)?;
        Ok((
            i,
            UncompressedFrameV4 {
                joint_hash,
                frame,
                padding,
            },
        ))
    }
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

#[derive(Debug, Clone)]
struct RawTrackV3 {
    track_name_bytes: [u8; 32],
    _flags: u32,
    pub frames: Vec<RawFrameV3>,
}

impl RawTrackV3 {
    pub fn parse(input: &[u8], frame_count: i32) -> IResult<&[u8], Self> {
        let (i, track_name_bytes) = take(32usize)(input)?;
        let (i, _flags) = le_u32(i)?;
        let (i, frames) = count(RawFrameV3::parse, frame_count as usize).parse(i)?;
        Ok((
            i,
            RawTrackV3 {
                track_name_bytes: track_name_bytes.try_into().unwrap(),
                _flags,
                frames,
            },
        ))
    }
}

#[derive(Debug, Clone)]
struct RawFrameV3 {
    pub rotation: Quat,
    pub translation: Vec3,
}

impl RawFrameV3 {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, rotation) = parse_quat(input)?;
        let (i, translation) = parse_vec3(i)?;
        Ok((
            i,
            RawFrameV3 {
                rotation,
                translation,
            },
        ))
    }
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

impl UncompressedDataV3 {
    pub fn parse(full_input: &[u8]) -> IResult<&[u8], Self> {
        let i = &full_input[12..];
        let (i, skeleton_id) = le_u32(i)?;
        let (i, track_count) = le_i32(i)?;
        let (i, frame_count) = le_i32(i)?;
        let (i, fps) = le_i32(i)?;

        let (i, tracks) = count(
            |input| RawTrackV3::parse(input, frame_count),
            track_count as usize,
        )
        .parse(i)?;

        let mut joint_frames = HashMap::with_capacity(track_count as usize);
        let palette_size = (track_count * frame_count) as usize;
        let mut quat_palette = Vec::with_capacity(palette_size);

        let mut vector_palette = Vec::with_capacity(palette_size + 1);
        vector_palette.push(Vec3::ONE);

        for (i, raw_track) in tracks.into_iter().enumerate() {
            let track_name = String::from_utf8_lossy(&raw_track.track_name_bytes)
                .trim_end_matches('\0')
                .to_string();

            let joint_hash = league_utils::hash_joint(&track_name);

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

        Ok((
            i,
            UncompressedDataV3 {
                skeleton_id,
                track_count,
                frame_count,
                fps,
                joint_frames,
                vector_palette,
                quat_palette,
            },
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UncompressedFrame {
    pub translation_id: u16,
    pub scale_id: u16,
    pub rotation_id: u16,
}

impl UncompressedFrame {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, translation_id) = le_u16(input)?;
        let (i, scale_id) = le_u16(i)?;
        let (i, rotation_id) = le_u16(i)?;
        Ok((
            i,
            UncompressedFrame {
                translation_id,
                scale_id,
                rotation_id,
            },
        ))
    }
}

const ONE_OVER_USHORT_MAX: f32 = 1.0 / 65535.0;
const ONE_OVER_SQRT_2: f32 = 1.0 / SQRT_2;

pub fn decompress_time(time: u16, duration: f32) -> f32 {
    time as f32 * ONE_OVER_USHORT_MAX * duration
}

pub fn decompress_vector3(value: &[u16; 3], min: &Vec3, max: &Vec3) -> Vec3 {
    let mut uncompressed = *max - *min;

    uncompressed.x *= value[0] as f32 * ONE_OVER_USHORT_MAX;
    uncompressed.y *= value[1] as f32 * ONE_OVER_USHORT_MAX;
    uncompressed.z *= value[2] as f32 * ONE_OVER_USHORT_MAX;

    uncompressed + *min
}

pub fn decompress_quat(value: [u16; 3]) -> Quat {
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

fn parse_vec3(input: &[u8]) -> IResult<&[u8], Vec3> {
    let (i, x) = le_f32(input)?;
    let (i, y) = le_f32(i)?;
    let (i, z) = le_f32(i)?;
    Ok((i, Vec3::new(x, y, z)))
}

fn parse_quat(input: &[u8]) -> IResult<&[u8], Quat> {
    let (i, x) = le_f32(input)?;
    let (i, y) = le_f32(i)?;
    let (i, z) = le_f32(i)?;
    let (i, w) = le_f32(i)?;
    Ok((i, Quat::from_xyzw(x, y, z, w)))
}
