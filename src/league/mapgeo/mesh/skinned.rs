use crate::league::BoundingBox;
use bevy::prelude::*;
use binrw::binread;

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct SkinnedMeshRange {
    #[br(count = 64, try_map = |bytes: Vec<u8>| {
        // 找到第一个 null 字符，如果有的话就截断
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8(bytes[..end].to_vec())
    })]
    pub name: String,
    pub start_vertex: u32,
    pub vertex_count: u32,
    pub start_index: u32,
    pub index_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinnedMeshVertex {
    Basic,
    Color,
    Tangent,
}

impl SkinnedMeshVertex {
    pub fn get_vertex_size(&self) -> u32 {
        match self {
            SkinnedMeshVertex::Basic => 52,
            SkinnedMeshVertex::Color => 56,
            SkinnedMeshVertex::Tangent => 72,
        }
    }
}

#[binread]
#[derive(Debug, Clone, Copy)]
#[br(little)]
pub struct BoundingSphere {
    pub center: [f32; 3],
    pub radius: f32,
}

#[derive(Debug)]
pub struct LeagueSkinnedMesh {
    pub major: u16,
    pub minor: u16,
    pub ranges: Vec<SkinnedMeshRange>,
    pub flags: Option<u32>,
    pub bounding_box: Option<BoundingBox>,
    pub bounding_sphere: Option<BoundingSphere>,
    pub index_count: u32,
    pub vertex_count: u32,
    pub vertex_declaration: SkinnedMeshVertex,
    pub index_buffer: Vec<u8>,
    pub vertex_buffer: Vec<u8>,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct LeagueSkinnedMeshInternal {
    #[br(magic = b"\x33\x22\x11\x00")]
    _magic: (),
    pub major: u16,
    pub minor: u16,
    #[br(assert((major == 0 || major == 2 || major == 4) && minor == 1, "无效的文件版本: {}.{}", major, minor))]
    _version_check: (),

    #[br(if(major == 0))]
    #[br(temp)]
    index_count_v0: Option<u32>,
    #[br(if(major == 0))]
    #[br(temp)]
    vertex_count_v0: Option<u32>,
    #[br(if(major > 0))]
    #[br(temp)]
    range_count: Option<u32>,
    #[br(count = range_count.unwrap_or(0))]
    pub ranges: Vec<SkinnedMeshRange>,
    #[br(if(major == 4))]
    pub flags: Option<u32>,
    #[br(if(major > 0))]
    #[br(temp)]
    index_count_v2_4: Option<u32>,
    #[br(if(major > 0))]
    #[br(temp)]
    vertex_count_v2_4: Option<u32>,
    #[br(if(major == 4))]
    #[br(temp)]
    vertex_size: Option<u32>,
    #[br(if(major == 4))]
    #[br(temp)]
    vertex_type_raw: Option<u32>,
    #[br(if(major == 4))]
    pub bounding_box: Option<BoundingBox>,
    #[br(if(major == 4))]
    pub bounding_sphere: Option<BoundingSphere>,

    #[br(calc = index_count_v0.or(index_count_v2_4).unwrap_or(0))]
    pub index_count: u32,
    #[br(calc = vertex_count_v0.or(vertex_count_v2_4).unwrap_or(0))]
    pub vertex_count: u32,
    #[br(calc = Self::parse_vertex_declaration(major, vertex_size, vertex_type_raw))]
    pub vertex_declaration: SkinnedMeshVertex,

    #[br(count = index_count * 2)]
    pub index_buffer: Vec<u8>,
    #[br(count = vertex_count * vertex_declaration.get_vertex_size())]
    pub vertex_buffer: Vec<u8>,
}

impl LeagueSkinnedMeshInternal {
    fn parse_vertex_declaration(
        major: u16,
        vertex_size: Option<u32>,
        vertex_type_raw: Option<u32>,
    ) -> SkinnedMeshVertex {
        if major != 4 {
            return SkinnedMeshVertex::Basic;
        }
        let size = vertex_size.expect("Version 4 must have vertex_size");
        let v_type = vertex_type_raw.expect("Version 4 must have vertex_type");
        match (size, v_type) {
            (52, 0) => SkinnedMeshVertex::Basic,
            (56, 1) => SkinnedMeshVertex::Color,
            (72, 2) => SkinnedMeshVertex::Tangent,
            _ => panic!("不支持的顶点格式: size={}, type={}", size, v_type),
        }
    }
}

impl From<LeagueSkinnedMeshInternal> for LeagueSkinnedMesh {
    fn from(internal: LeagueSkinnedMeshInternal) -> Self {
        let mut final_ranges = internal.ranges;

        if internal.major == 0 {
            final_ranges = vec![SkinnedMeshRange {
                name: "".to_string(),
                start_vertex: 0,
                vertex_count: internal.vertex_count,
                start_index: 0,
                index_count: internal.index_count,
            }];
        }

        Self {
            major: internal.major,
            minor: internal.minor,
            ranges: final_ranges,
            flags: internal.flags,
            bounding_box: internal.bounding_box,
            bounding_sphere: internal.bounding_sphere,
            index_count: internal.index_count,
            vertex_count: internal.vertex_count,
            vertex_declaration: internal.vertex_declaration,
            index_buffer: internal.index_buffer,
            vertex_buffer: internal.vertex_buffer,
        }
    }
}
