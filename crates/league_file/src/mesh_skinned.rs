use bevy::prelude::*;
use league_utils::BoundingBox;
use nom::bytes::complete::take;
use nom::multi::count;
use nom::number::complete::{le_f32, le_u16, le_u32};
use nom::{IResult, Parser};

#[derive(Debug, Clone)]
pub struct SkinnedMeshRange {
    pub name: String,
    pub start_vertex: u32,
    pub vertex_count: u32,
    pub start_index: u32,
    pub index_count: u32,
}

impl SkinnedMeshRange {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, name_bytes) = take(64usize)(input)?;
        let end = name_bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(name_bytes.len());
        let name = String::from_utf8_lossy(&name_bytes[..end]).to_string();

        let (i, start_vertex) = le_u32(i)?;
        let (i, vertex_count) = le_u32(i)?;
        let (i, start_index) = le_u32(i)?;
        let (i, index_count) = le_u32(i)?;

        Ok((
            i,
            SkinnedMeshRange {
                name,
                start_vertex,
                vertex_count,
                start_index,
                index_count,
            },
        ))
    }
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

#[derive(Debug, Clone, Copy)]
pub struct BoundingSphere {
    pub center: [f32; 3],
    pub radius: f32,
}

impl BoundingSphere {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, c0) = le_f32(input)?;
        let (i, c1) = le_f32(i)?;
        let (i, c2) = le_f32(i)?;
        let (i, radius) = le_f32(i)?;
        Ok((
            i,
            BoundingSphere {
                center: [c0, c1, c2],
                radius,
            },
        ))
    }
}

#[derive(Debug, Asset, TypePath)]
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

impl LeagueSkinnedMesh {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, _) = take(4usize)(input)?; // magic: \x33\x22\x11\x00
        let (i, major) = le_u16(i)?;
        let (i, minor) = le_u16(i)?;

        if !((major == 0 || major == 2 || major == 4) && minor == 1) {
            panic!("Invalid file version: {}.{}", major, minor);
        }

        let mut current_i = i;
        let index_count;
        let vertex_count;
        let mut ranges = Vec::new();
        let mut flags = None;
        let mut vertex_size = None;
        let mut vertex_type_raw = None;
        let mut bounding_box = None;
        let mut bounding_sphere = None;

        if major == 0 {
            let (i_next, ic) = le_u32(current_i)?;
            let (i_next, vc) = le_u32(i_next)?;
            index_count = ic;
            vertex_count = vc;
            current_i = i_next;
        } else {
            let (i_next, range_count) = le_u32(current_i)?;
            let (i_next, r) = count(SkinnedMeshRange::parse, range_count as usize).parse(i_next)?;
            ranges = r;
            current_i = i_next;

            if major == 4 {
                let (i_next, f) = le_u32(current_i)?;
                flags = Some(f);
                current_i = i_next;
            }

            let (i_next, ic) = le_u32(current_i)?;
            let (i_next, vc) = le_u32(i_next)?;
            index_count = ic;
            vertex_count = vc;
            current_i = i_next;

            if major == 4 {
                let (i_next, vs) = le_u32(current_i)?;
                let (i_next, vt) = le_u32(i_next)?;
                let (i_next, bb) = BoundingBox::parse(i_next)?;
                let (i_next, bs) = BoundingSphere::parse(i_next)?;
                vertex_size = Some(vs);
                vertex_type_raw = Some(vt);
                bounding_box = Some(bb);
                bounding_sphere = Some(bs);
                current_i = i_next;
            }
        }

        let vertex_declaration =
            Self::parse_vertex_declaration(major, vertex_size, vertex_type_raw);
        let (i, index_buffer) = take((index_count * 2) as usize)(current_i)?;
        let (i, vertex_buffer) =
            take((vertex_count * vertex_declaration.get_vertex_size()) as usize)(i)?;

        Ok((
            i,
            LeagueSkinnedMesh {
                major,
                minor,
                ranges,
                flags,
                bounding_box,
                bounding_sphere,
                index_count,
                vertex_count,
                vertex_declaration,
                index_buffer: index_buffer.to_vec(),
                vertex_buffer: vertex_buffer.to_vec(),
            },
        ))
    }

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
