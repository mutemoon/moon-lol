use bevy::asset::Asset;
use bevy::math::{Mat4, Quat, Vec3, Vec4};
use bevy::reflect::TypePath;
use binrw::io::{Read, Seek, SeekFrom};
use binrw::{binread, BinRead};
use binrw::{prelude::*, Endian};
use league_utils::{parse_quat, parse_vec3};

const FORMAT_TOKEN: u32 = 0x22FD4FC3;

#[binread]
#[derive(Debug, Asset, TypePath)]
#[br(little)]
pub struct LeagueSkeleton {
    #[br(temp)]
    _file_size: u32,

    #[br(temp)]
    format_token: u32,

    #[br(args(format_token))]
    #[br(map = |kind: SkeletonDataKind| kind.into())]
    pub modern_data: SkeletonData,
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

#[binread]
#[br(import(format_token: u32))]
enum SkeletonDataKind {
    #[br(pre_assert(format_token == FORMAT_TOKEN))]
    Modern(ModernSkeletonData),

    Legacy(LegacySkeletonData),
}

impl From<SkeletonDataKind> for SkeletonData {
    fn from(kind: SkeletonDataKind) -> Self {
        match kind {
            SkeletonDataKind::Modern(modern) => {
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
            SkeletonDataKind::Legacy(legacy) => SkeletonData {
                flags: 0,
                name: String::new(),
                asset_name: String::new(),
                joints: legacy.joints,
                influences: legacy.influences,
            },
        }
    }
}

#[binread]
#[br(little)]
struct ModernSkeletonData {
    #[br(temp)]
    version: u32,
    #[br(assert(version == 0, "Invalid skeleton version: {}", version))]
    pub flags: u16,
    #[br(temp)]
    joint_count: u16,
    #[br(temp)]
    influences_count: u32,

    #[br(temp)]
    joints_offset: i32,
    _joint_indices_offset: i32,
    #[br(temp)]
    influences_offset: i32,
    #[br(temp)]
    name_offset: i32,
    #[br(temp)]
    asset_name_offset: i32,
    _bone_names_offset: i32,

    #[br(count = 5)]
    _reserved: Vec<i32>,

    #[br(
        seek_before = SeekFrom::Start(name_offset as u64),
        if(name_offset > 0),
        parse_with = read_null_terminated_string,
        restore_position
    )]
    pub name: String,

    #[br(
        seek_before = SeekFrom::Start(asset_name_offset as u64),
        if(asset_name_offset > 0),
        parse_with = read_null_terminated_string,
        restore_position
    )]
    pub asset_name: String,

    #[br(
        seek_before = SeekFrom::Start(joints_offset as u64),
        count = joint_count,
        if(joints_offset > 0)
    )]
    pub joints: Vec<RigResourceJoint>,

    #[br(
        seek_before = SeekFrom::Start(influences_offset as u64),
        count = influences_count,
        if(influences_offset > 0)
    )]
    pub influences: Vec<i16>,
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct RigResourceJoint {
    pub flags: u16,
    pub id: i16,
    pub parent_id: i16,
    #[br(temp)]
    _padding: u16,
    #[br(temp)]
    _name_hash: u32,
    pub radius: f32,

    #[br(temp, map = parse_vec3)]
    local_translation: Vec3,
    #[br(temp, map = parse_vec3)]
    local_scale: Vec3,
    #[br(temp, map = parse_quat)]
    local_rotation: Quat,
    #[br(temp, map = parse_vec3)]
    inverse_bind_translation: Vec3,
    #[br(temp, map = parse_vec3)]
    inverse_bind_scale: Vec3,
    #[br(temp, map = parse_quat)]
    inverse_bind_rotation: Quat,

    #[br(map = |rs: RelativeString| rs.0)]
    pub name: String,

    #[br(calc = Mat4::from_scale_rotation_translation(local_scale, local_rotation, local_translation))]
    pub local_transform: Mat4,

    #[br(calc = Mat4::from_scale_rotation_translation(inverse_bind_scale, inverse_bind_rotation, inverse_bind_translation))]
    pub inverse_bind_transform: Mat4,
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct LegacySkeletonData {
    #[br(seek_before = SeekFrom::Start(0), magic = b"r3d2sklt")]
    _magic: (),

    #[br(temp)]
    version: u32,
    #[br(assert(version == 1 || version == 2, "Invalid legacy skeleton version: {}", version))]
    _skeleton_id: u32,
    #[br(temp)]
    joint_count: u32,

    #[br(temp)]
    #[br(count = joint_count)]
    legacy_joints: Vec<RigResourceLegacyJoint>,

    #[br(temp)]
    #[br(if(version == 2))]
    #[br(parse_with = parse_legacy_influences)]
    influences_v2: Vec<i16>,

    #[br(calc = Self::calculate_joints(&legacy_joints))]
    pub joints: Vec<Joint>,

    #[br(calc = Self::calculate_influences(version, joint_count, &influences_v2))]
    pub influences: Vec<i16>,
}

impl LegacySkeletonData {
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

    fn calculate_influences(version: u32, joint_count: u32, influences_v2: &[i16]) -> Vec<i16> {
        if version == 2 {
            influences_v2.to_vec()
        } else {
            (0..joint_count as i16).collect()
        }
    }
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct RigResourceLegacyJoint {
    #[br(count = 32, map = |bytes: Vec<u8>| String::from_utf8_lossy(&bytes).trim_end_matches('\0').to_string())]
    pub name: String,
    pub parent_id: i32,
    pub radius: f32,
    #[br(map = |data: LegacyTransformData| data.into())]
    pub global_transform: Mat4,
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct LegacyTransformData {
    col0: [f32; 4],
    col1: [f32; 4],
    col2: [f32; 4],
}

impl From<LegacyTransformData> for Mat4 {
    fn from(data: LegacyTransformData) -> Self {
        Mat4::from_cols(
            data.col0.into(),
            data.col1.into(),
            data.col2.into(),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }
}

fn parse_legacy_influences<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    _: (),
) -> BinResult<Vec<i16>> {
    let count = u32::read_options(reader, endian, ())?;
    let mut influences = Vec::with_capacity(count as usize);
    for _ in 0..count {
        influences.push(u32::read_options(reader, endian, ())? as i16);
    }
    Ok(influences)
}

struct RelativeString(pub String);

impl BinRead for RelativeString {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let base_pos = reader.stream_position()?;
        let offset = i32::read_options(reader, endian, ())?;
        let after_offset_pos = reader.stream_position()?;

        let string_pos = (base_pos as i64 + offset as i64) as u64;

        reader.seek(SeekFrom::Start(string_pos))?;
        let value = read_null_terminated_string(reader, endian, ())?;
        reader.seek(SeekFrom::Start(after_offset_pos))?;

        Ok(RelativeString(value))
    }
}

fn read_null_terminated_string<R: Read>(reader: &mut R, _: Endian, _: ()) -> BinResult<String> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        if byte[0] == 0 {
            break;
        }
        bytes.push(byte[0]);
    }
    Ok(String::from_utf8_lossy(&bytes).to_string())
}
