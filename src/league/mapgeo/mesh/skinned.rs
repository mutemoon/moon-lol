use crate::league::BoundingBox;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
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

impl LeagueSkinnedMesh {
    pub fn to_bevy_mesh(&self, submesh_index: usize) -> Option<Mesh> {
        let range = self.ranges.get(submesh_index)?;

        let vertex_size = self.vertex_declaration.get_vertex_size() as usize;
        if vertex_size == 0 {
            return None;
        }

        let vertex_start_byte = range.start_vertex as usize * vertex_size;
        let vertex_end_byte = vertex_start_byte + (range.vertex_count as usize * vertex_size);
        let vertex_data_slice = self.vertex_buffer.get(vertex_start_byte..vertex_end_byte)?;

        let capacity = range.vertex_count as usize;
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(capacity);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(capacity);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(capacity);
        let mut joint_indices: Vec<[u16; 4]> = Vec::with_capacity(capacity);
        let mut joint_weights: Vec<[f32; 4]> = Vec::with_capacity(capacity);

        let mut colors: Option<Vec<[f32; 4]>> = None;
        let mut tangents: Option<Vec<[f32; 4]>> = None;

        if self.vertex_declaration != SkinnedMeshVertex::Basic {
            colors = Some(Vec::with_capacity(capacity));
            if self.vertex_declaration == SkinnedMeshVertex::Tangent {
                tangents = Some(Vec::with_capacity(capacity));
            }
        }

        for v_chunk in vertex_data_slice.chunks_exact(vertex_size) {
            let mut offset = 0;

            let x_pos = f32::from_le_bytes(v_chunk[offset..offset + 4].try_into().unwrap());
            let y_pos = f32::from_le_bytes(v_chunk[offset + 4..offset + 8].try_into().unwrap());
            let z_pos = f32::from_le_bytes(v_chunk[offset + 8..offset + 12].try_into().unwrap());
            positions.push([x_pos, y_pos, z_pos]);
            offset += 12;

            let j_indices_u8: [u8; 4] = v_chunk[offset..offset + 4].try_into().unwrap();
            joint_indices.push([
                j_indices_u8[0] as u16,
                j_indices_u8[1] as u16,
                j_indices_u8[2] as u16,
                j_indices_u8[3] as u16,
            ]);
            offset += 4;

            let weights: [f32; 4] = [
                f32::from_le_bytes(v_chunk[offset..offset + 4].try_into().unwrap()),
                f32::from_le_bytes(v_chunk[offset + 4..offset + 8].try_into().unwrap()),
                f32::from_le_bytes(v_chunk[offset + 8..offset + 12].try_into().unwrap()),
                f32::from_le_bytes(v_chunk[offset + 12..offset + 16].try_into().unwrap()),
            ];
            joint_weights.push(weights);
            offset += 16;

            let x_norm = f32::from_le_bytes(v_chunk[offset..offset + 4].try_into().unwrap());
            let y_norm = f32::from_le_bytes(v_chunk[offset + 4..offset + 8].try_into().unwrap());
            let z_norm = f32::from_le_bytes(v_chunk[offset + 8..offset + 12].try_into().unwrap());
            normals.push([x_norm, y_norm, z_norm]);
            offset += 12;

            let u = f32::from_le_bytes(v_chunk[offset..offset + 4].try_into().unwrap());
            let v = f32::from_le_bytes(v_chunk[offset + 4..offset + 8].try_into().unwrap());
            uvs.push([u, v]);
            offset += 8;

            if let Some(colors_vec) = colors.as_mut() {
                let color_u8: [u8; 4] = v_chunk[offset..offset + 4].try_into().unwrap();
                colors_vec.push([
                    color_u8[2] as f32 / 255.0,
                    color_u8[1] as f32 / 255.0,
                    color_u8[0] as f32 / 255.0,
                    color_u8[3] as f32 / 255.0,
                ]);
                offset += 4;
            }

            if let Some(tangents_vec) = tangents.as_mut() {
                let tan_x = f32::from_le_bytes(v_chunk[offset..offset + 4].try_into().unwrap());
                let tan_y = f32::from_le_bytes(v_chunk[offset + 4..offset + 8].try_into().unwrap());
                let tan_z =
                    f32::from_le_bytes(v_chunk[offset + 8..offset + 12].try_into().unwrap());
                let tan_w =
                    f32::from_le_bytes(v_chunk[offset + 12..offset + 16].try_into().unwrap());

                tangents_vec.push([tan_x, tan_y, tan_z, tan_w]);
            }
        }

        let index_start_byte = range.start_index as usize * 2;
        let index_end_byte = index_start_byte + (range.index_count as usize * 2);
        let index_data_slice = self.index_buffer.get(index_start_byte..index_end_byte)?;

        let local_indices: Vec<u16> = index_data_slice
            .chunks_exact(2)
            .map(|bytes| u16::from_le_bytes(bytes.try_into().unwrap()))
            .map(|global_index| global_index - range.start_vertex as u16)
            .collect();

        let mut bevy_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        bevy_mesh.insert_attribute(
            Mesh::ATTRIBUTE_JOINT_INDEX,
            VertexAttributeValues::Uint16x4(joint_indices),
        );
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, joint_weights);

        if let Some(colors_data) = colors {
            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors_data);
        }
        if let Some(tangents_data) = tangents {
            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents_data);
        }

        bevy_mesh.insert_indices(Indices::U16(local_indices));

        Some(bevy_mesh)
    }
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
