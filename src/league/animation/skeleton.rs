use crate::league::{BinQuat, BinVec3};
use bevy::math::{Mat4, Quat, Vec3, Vec4};
use binrw::io::{Read, Seek, SeekFrom};
use binrw::{binread, BinRead};
use binrw::{prelude::*, Endian};

// ===================================================================================
// 1. 常量和公共数据结构 (Public API)
// ===================================================================================

const FORMAT_TOKEN: u32 = 0x22FD4FC3; // FNV hash of the format token string

/// 顶层骨骼文件结构体。
/// 这是解析的入口点。
#[binread]
#[derive(Debug)]
#[br(little)]
pub struct LeagueSkeleton {
    #[br(temp)]
    _file_size: u32,

    #[br(temp)]
    format_token: u32,

    // 使用 args 传递 format_token，并用 map 将解析结果转换为最终的 SkeletonData
    #[br(args(format_token))]
    #[br(map = |kind: SkeletonDataKind| kind.into())]
    pub modern_data: SkeletonData,
}

/// 统一的骨骼数据，无论是从现代格式还是旧版格式解析而来。
#[derive(Debug)]
pub struct SkeletonData {
    pub flags: u16,
    pub name: String,
    pub asset_name: String,
    pub joints: Vec<Joint>,
    pub influences: Vec<i16>,
}

/// 最终在应用程序中使用的关节数据结构。
#[derive(Debug, Clone)]
pub struct Joint {
    pub name: String,
    pub flags: u16,
    pub id: i16,
    pub parent_id: i16,
    pub radius: f32,
    pub local_transform: Mat4,
    pub inverse_bind_transform: Mat4,
}

// ===================================================================================
// 2. 解析器实现 (Parser Implementation)
// ===================================================================================

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ 条件解析 ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// 这是一个中间枚举，用于根据 `format_token` 进行条件解析。
#[binread]
#[br(import(format_token: u32))]
enum SkeletonDataKind {
    /// 如果 format_token 匹配，则解析为现代骨骼
    #[br(pre_assert(format_token == FORMAT_TOKEN))]
    Modern(ModernSkeletonData),

    /// 否则，解析为旧版骨骼
    Legacy(LegacySkeletonData),
}

/// 实现 From trait，将中间解析结果 `SkeletonDataKind` 转换为最终的公共类型 `SkeletonData`。
/// 这样可以统一不同文件格式的数据。
impl From<SkeletonDataKind> for SkeletonData {
    fn from(kind: SkeletonDataKind) -> Self {
        match kind {
            SkeletonDataKind::Modern(modern) => {
                // 将 RigResourceJoint 转换为应用程序使用的 Joint
                let joints = modern
                    .joints
                    .into_iter()
                    .map(|j| Joint {
                        name: j.name,
                        flags: j.flags,
                        id: j.id,
                        parent_id: j.parent_id,
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
                name: String::new(), // 旧版格式没有骨骼名称和资产名称
                asset_name: String::new(),
                joints: legacy.joints,
                influences: legacy.influences,
            },
        }
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~ 现代格式解析 (Modern Format) ~~~~~~~~~~~~~~~~~~~~~~~~~

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

    // Offsets
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

    // Transform components
    #[br(temp)]
    local_translation: BinVec3,
    #[br(temp)]
    local_scale: BinVec3,
    #[br(temp)]
    local_rotation: BinQuat,
    #[br(temp)]
    inverse_bind_translation: BinVec3,
    #[br(temp)]
    inverse_bind_scale: BinVec3,
    #[br(temp)]
    inverse_bind_rotation: BinQuat,

    // 使用自定义的 RelativeString 类型来处理相对偏移
    #[br(map = |rs: RelativeString| rs.0)]
    pub name: String,

    // Calculated fields
    #[br(calc = create_transform_matrix(local_scale.0, local_rotation.0, local_translation.0))]
    pub local_transform: Mat4,

    #[br(calc = create_transform_matrix(inverse_bind_scale.0, inverse_bind_rotation.0, inverse_bind_translation.0))]
    pub inverse_bind_transform: Mat4,
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~ 旧版格式解析 (Legacy Format) ~~~~~~~~~~~~~~~~~~~~~~~~~~

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

    // 使用 calc 在解析后计算最终的 joints 和 influences
    #[br(calc = Self::calculate_joints(&legacy_joints))]
    pub joints: Vec<Joint>,

    #[br(calc = Self::calculate_influences(version, joint_count, &influences_v2))]
    pub influences: Vec<i16>,
}

impl LegacySkeletonData {
    /// 从旧版关节数据计算出符合现代格式的关节列表（包含局部变换矩阵）。
    fn calculate_joints(legacy_joints: &[RigResourceLegacyJoint]) -> Vec<Joint> {
        // 验证关节必须按层级顺序排列
        for (i, joint) in legacy_joints.iter().enumerate() {
            if i as i32 <= joint.parent_id {
                // 在生产代码中，应返回一个 Result 而不是 panic
                panic!("Joints must be ordered hierarchically");
            }
        }

        // 计算每个关节的局部变换矩阵
        let local_transforms: Vec<Mat4> = legacy_joints
            .iter()
            .map(|joint| {
                if joint.parent_id == -1 {
                    joint.global_transform
                } else {
                    let parent_global = legacy_joints[joint.parent_id as usize].global_transform;
                    // local_transform = global_transform * inverse(parent_global_transform)
                    joint.global_transform * parent_global.inverse()
                }
            })
            .collect();

        // 转换为统一的 Joint 格式
        legacy_joints
            .iter()
            .enumerate()
            .map(|(i, legacy_joint)| Joint {
                name: legacy_joint.name.clone(),
                flags: 0,
                id: i as i16,
                parent_id: legacy_joint.parent_id as i16,
                radius: legacy_joint.radius,
                local_transform: local_transforms[i],
                inverse_bind_transform: legacy_joint.global_transform.inverse(),
            })
            .collect()
    }

    /// 根据版本计算 influences 列表。
    fn calculate_influences(version: u32, joint_count: u32, influences_v2: &[i16]) -> Vec<i16> {
        if version == 2 {
            influences_v2.to_vec()
        } else {
            // Version 1
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

/// 解析旧版格式（版本2）中的 influences 列表。
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

// ===================================================================================
// 3. 辅助类型和函数 (Helpers)
// ===================================================================================

/// 一个自定义类型，用于封装“读取相对偏移 -> 跳转 -> 读取字符串 -> 恢复”的逻辑。
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

/// 从字节流中读取一个以 null (`\0`) 结尾的字符串。
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

/// 根据缩放、旋转和平移分量创建一个 4x4 变换矩阵。
fn create_transform_matrix(scale: Vec3, rotation: Quat, translation: Vec3) -> Mat4 {
    Mat4::from_scale_rotation_translation(scale, rotation, translation)
}
