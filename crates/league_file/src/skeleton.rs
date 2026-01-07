use bevy::asset::Asset;
use bevy::math::{Mat4, Quat, Vec3, Vec4};
use bevy::reflect::TypePath;
use nom::bytes::complete::take;
use nom::multi::count;
use nom::number::complete::{le_f32, le_i16, le_i32, le_u16, le_u32, le_u8};
use nom::{IResult, Parser};

pub const FORMAT_TOKEN: u32 = 0x22FD4FC3;

#[derive(Debug, Asset, TypePath)]
pub struct LeagueSkeleton {
    pub modern_data: SkeletonData,
}

impl LeagueSkeleton {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, _file_size) = le_u32(input)?;
        let (i, format_token) = le_u32(i)?;

        if format_token == FORMAT_TOKEN {
            let (_, modern) = ModernSkeletonData::parse(input)?;
            Ok((
                i,
                LeagueSkeleton {
                    modern_data: modern.into(),
                },
            ))
        } else {
            let (_, legacy) = LegacySkeletonData::parse(input)?;
            Ok((
                i,
                LeagueSkeleton {
                    modern_data: legacy.into(),
                },
            ))
        }
    }
}

#[derive(Debug)]
pub struct SkeletonData {
    pub flags: u16,
    pub name: String,
    pub asset_name: String,
    pub joints: Vec<Joint>,
    pub influences: Vec<i16>,
}

#[derive(Debug, Clone)]
pub struct Joint {
    pub name: String,
    pub flags: u16,
    pub index: i16,
    pub parent_index: i16,
    pub radius: f32,
    pub local_transform: Mat4,
    pub inverse_bind_transform: Mat4,
}

struct ModernSkeletonData {
    pub flags: u16,
    pub name: String,
    pub asset_name: String,
    pub joints: Vec<RigResourceJoint>,
    pub influences: Vec<i16>,
}

impl ModernSkeletonData {
    pub fn parse(full_input: &[u8]) -> IResult<&[u8], Self> {
        let i = &full_input[8..];
        let (i, version) = le_u32(i)?;
        if version != 0 {
            panic!("Invalid skeleton version: {}", version);
        }
        let (i, flags) = le_u16(i)?;
        let (i, joint_count) = le_u16(i)?;
        let (i, influences_count) = le_u32(i)?;

        let (i, joints_offset) = le_i32(i)?;
        let (i, _joint_indices_offset) = le_i32(i)?;
        let (i, influences_offset) = le_i32(i)?;
        let (i, name_offset) = le_i32(i)?;
        let (i, asset_name_offset) = le_i32(i)?;
        let (i, _bone_names_offset) = le_i32(i)?;

        let (i, _reserved) = count(le_i32, 5).parse(i)?;

        let name = if name_offset > 0 {
            read_null_terminated_string(&full_input[name_offset as usize..])?.1
        } else {
            String::new()
        };

        let asset_name = if asset_name_offset > 0 {
            read_null_terminated_string(&full_input[asset_name_offset as usize..])?.1
        } else {
            String::new()
        };

        let joints = if joints_offset > 0 {
            count(
                |input| RigResourceJoint::parse(input, full_input),
                joint_count as usize,
            )
            .parse(&full_input[joints_offset as usize..])?
            .1
        } else {
            Vec::new()
        };

        let influences = if influences_offset > 0 {
            count(le_i16, influences_count as usize)
                .parse(&full_input[influences_offset as usize..])?
                .1
        } else {
            Vec::new()
        };

        Ok((
            i,
            ModernSkeletonData {
                flags,
                name,
                asset_name,
                joints,
                influences,
            },
        ))
    }
}

impl From<ModernSkeletonData> for SkeletonData {
    fn from(modern: ModernSkeletonData) -> Self {
        let joints = modern
            .joints
            .into_iter()
            .map(|j| Joint {
                name: j.name,
                flags: j.flags,
                index: j.id,
                parent_index: j.parent_id,
                radius: j.radius,
                local_transform: j.local_transform,
                inverse_bind_transform: j.inverse_bind_transform,
            })
            .collect();

        SkeletonData {
            flags: modern.flags,
            name: modern.name,
            asset_name: modern.asset_name,
            joints,
            influences: modern.influences,
        }
    }
}

struct RigResourceJoint {
    pub flags: u16,
    pub id: i16,
    pub parent_id: i16,
    pub radius: f32,
    pub name: String,
    pub local_transform: Mat4,
    pub inverse_bind_transform: Mat4,
}

impl RigResourceJoint {
    pub fn parse<'a>(input: &'a [u8], full_input: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, flags) = le_u16(input)?;
        let (i, id) = le_i16(i)?;
        let (i, parent_id) = le_i16(i)?;
        let (i, _padding) = le_u16(i)?;
        let (i, _name_hash) = le_u32(i)?;
        let (i, radius) = le_f32(i)?;

        let (i, local_translation) = parse_vec3(i)?;
        let (i, local_scale) = parse_vec3(i)?;
        let (i, local_rotation) = parse_quat(i)?;
        let (i, inverse_bind_translation) = parse_vec3(i)?;
        let (i, inverse_bind_scale) = parse_vec3(i)?;
        let (i, inverse_bind_rotation) = parse_quat(i)?;

        let (i, name_offset) = le_i32(i)?;
        // Relative string: base_pos + offset
        // base_pos is the position of the offset itself.
        // In our case, the offset was at input - 4.
        let base_pos = (input.as_ptr() as usize - full_input.as_ptr() as usize)
            + (i.as_ptr() as usize - input.as_ptr() as usize)
            - 4;
        let string_pos = (base_pos as i64 + name_offset as i64) as usize;
        let name = read_null_terminated_string(&full_input[string_pos..])?.1;

        Ok((
            i,
            RigResourceJoint {
                flags,
                id,
                parent_id,
                radius,
                name,
                local_transform: Mat4::from_scale_rotation_translation(
                    local_scale,
                    local_rotation,
                    local_translation,
                ),
                inverse_bind_transform: Mat4::from_scale_rotation_translation(
                    inverse_bind_scale,
                    inverse_bind_rotation,
                    inverse_bind_translation,
                ),
            },
        ))
    }
}

struct LegacySkeletonData {
    pub joints: Vec<Joint>,
    pub influences: Vec<i16>,
}

impl LegacySkeletonData {
    pub fn parse(full_input: &[u8]) -> IResult<&[u8], Self> {
        let (i, magic) = take(8usize)(full_input)?;
        if magic != b"r3d2sklt" {
            panic!("Invalid legacy skeleton magic");
        }
        let (i, version) = le_u32(i)?;
        if version != 1 && version != 2 {
            panic!("Invalid legacy skeleton version: {}", version);
        }
        let (i, _skeleton_id) = le_u32(i)?;
        let (i, joint_count) = le_u32(i)?;

        let (i, legacy_joints) =
            count(RigResourceLegacyJoint::parse, joint_count as usize).parse(i)?;

        let (i, influences) = if version == 2 {
            let (i, count_val) = le_u32(i)?;
            let (i, influences_raw) = count(le_u32, count_val as usize).parse(i)?;
            (i, influences_raw.into_iter().map(|v| v as i16).collect())
        } else {
            (i, (0..joint_count as i16).collect())
        };

        let joints = Self::calculate_joints(&legacy_joints);

        Ok((i, LegacySkeletonData { joints, influences }))
    }

    fn calculate_joints(legacy_joints: &[RigResourceLegacyJoint]) -> Vec<Joint> {
        for (i, joint) in legacy_joints.iter().enumerate() {
            if i as i32 <= joint.parent_id {
                panic!("Joints must be ordered hierarchically");
            }
        }

        let local_transforms: Vec<Mat4> = legacy_joints
            .iter()
            .map(|joint| {
                if joint.parent_id == -1 {
                    joint.global_transform
                } else {
                    let parent_global = legacy_joints[joint.parent_id as usize].global_transform;
                    joint.global_transform * parent_global.inverse()
                }
            })
            .collect();

        legacy_joints
            .iter()
            .enumerate()
            .map(|(i, legacy_joint)| Joint {
                name: legacy_joint.name.clone(),
                flags: 0,
                index: i as i16,
                parent_index: legacy_joint.parent_id as i16,
                radius: legacy_joint.radius,
                local_transform: local_transforms[i],
                inverse_bind_transform: legacy_joint.global_transform.inverse(),
            })
            .collect()
    }
}

impl From<LegacySkeletonData> for SkeletonData {
    fn from(legacy: LegacySkeletonData) -> Self {
        SkeletonData {
            flags: 0,
            name: String::new(),
            asset_name: String::new(),
            joints: legacy.joints,
            influences: legacy.influences,
        }
    }
}

struct RigResourceLegacyJoint {
    pub name: String,
    pub parent_id: i32,
    pub radius: f32,
    pub global_transform: Mat4,
}

impl RigResourceLegacyJoint {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, name_bytes) = take(32usize)(input)?;
        let name = String::from_utf8_lossy(name_bytes)
            .trim_end_matches('\0')
            .to_string();
        let (i, parent_id) = le_i32(i)?;
        let (i, radius) = le_f32(i)?;
        let (i, col0) = count(le_f32, 4).parse(i)?;
        let (i, col1) = count(le_f32, 4).parse(i)?;
        let (i, col2) = count(le_f32, 4).parse(i)?;

        let global_transform = Mat4::from_cols(
            Vec4::from_slice(&col0),
            Vec4::from_slice(&col1),
            Vec4::from_slice(&col2),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );

        Ok((
            i,
            RigResourceLegacyJoint {
                name,
                parent_id,
                radius,
                global_transform,
            },
        ))
    }
}

fn read_null_terminated_string(input: &[u8]) -> IResult<&[u8], String> {
    let mut bytes = Vec::new();
    let mut current_input = input;
    loop {
        let (i, byte) = le_u8(current_input)?;
        if byte == 0 {
            current_input = i;
            break;
        }
        bytes.push(byte);
        current_input = i;
    }
    Ok((current_input, String::from_utf8_lossy(&bytes).to_string()))
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
