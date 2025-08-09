use crate::league::BoundingBox;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use binrw::binread;

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct SkinnedMeshRange {
    #[br(count = 64, try_map = |bytes: Vec<u8>| String::from_utf8(bytes))]
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

// ---- 新增部分: 干净的公开结构体和内部解析结构体 ----

/// 最终提供给用户的、干净的蒙皮网格结构体。
/// 它不包含任何 binrw 属性。
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
    /// 将此蒙皮网格中的特定子网格转换为 Bevy `Mesh`。
    ///
    /// # 参数
    /// * `submesh_index` - 要转换的子网格在 `ranges` Vec 中的索引。
    ///
    /// # 返回
    /// * `Some(Mesh)` - 如果成功创建了 Bevy 网格。
    /// * `None` - 如果 `submesh_index` 无效或数据切片失败。
    pub fn to_bevy_mesh(&self, submesh_index: usize) -> Option<Mesh> {
        // 1. 根据索引获取对应的子网格范围
        let range = self.ranges.get(submesh_index)?;

        // 2. 确定顶点结构和大小
        let vertex_size = self.vertex_declaration.get_vertex_size() as usize;
        if vertex_size == 0 {
            return None; // 无效的顶点大小
        }

        // 3. 获取此子网格对应的顶点数据切片
        let vertex_start_byte = range.start_vertex as usize * vertex_size;
        let vertex_end_byte = vertex_start_byte + (range.vertex_count as usize * vertex_size);
        let vertex_data_slice = self.vertex_buffer.get(vertex_start_byte..vertex_end_byte)?;

        // 4. 预分配用于存储解析后属性的 Vec
        let capacity = range.vertex_count as usize;
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(capacity);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(capacity);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(capacity);
        let mut joint_indices: Vec<[u16; 4]> = Vec::with_capacity(capacity);
        let mut joint_weights: Vec<[f32; 4]> = Vec::with_capacity(capacity);

        // 可选的顶点属性
        let mut colors: Option<Vec<[f32; 4]>> = None;
        let mut tangents: Option<Vec<[f32; 4]>> = None;

        if self.vertex_declaration != SkinnedMeshVertex::Basic {
            colors = Some(Vec::with_capacity(capacity));
            if self.vertex_declaration == SkinnedMeshVertex::Tangent {
                tangents = Some(Vec::with_capacity(capacity));
            }
        }

        // 5. 遍历并解析每个顶点
        for v_chunk in vertex_data_slice.chunks_exact(vertex_size) {
            let mut offset = 0;

            // --- 修正点 1: 调整顶点属性的解析顺序以匹配实际布局 ---

            // 位置 (12 bytes)
            let x_pos = f32::from_le_bytes(v_chunk[offset..offset + 4].try_into().unwrap());
            let y_pos = f32::from_le_bytes(v_chunk[offset + 4..offset + 8].try_into().unwrap());
            let z_pos = f32::from_le_bytes(v_chunk[offset + 8..offset + 12].try_into().unwrap());
            positions.push([x_pos, y_pos, z_pos]); // 翻转Z轴以适应Bevy的右手坐标系
            offset += 12;

            // 骨骼索引 (4 bytes, u8 -> u16)
            let j_indices_u8: [u8; 4] = v_chunk[offset..offset + 4].try_into().unwrap();
            joint_indices.push([
                j_indices_u8[0] as u16,
                j_indices_u8[1] as u16,
                j_indices_u8[2] as u16,
                j_indices_u8[3] as u16,
            ]);
            offset += 4;

            // 骨骼权重 (16 bytes)
            let weights: [f32; 4] = [
                f32::from_le_bytes(v_chunk[offset..offset + 4].try_into().unwrap()),
                f32::from_le_bytes(v_chunk[offset + 4..offset + 8].try_into().unwrap()),
                f32::from_le_bytes(v_chunk[offset + 8..offset + 12].try_into().unwrap()),
                f32::from_le_bytes(v_chunk[offset + 12..offset + 16].try_into().unwrap()),
            ];
            joint_weights.push(weights);
            offset += 16;

            // 法线 (12 bytes)
            let x_norm = f32::from_le_bytes(v_chunk[offset..offset + 4].try_into().unwrap());
            let y_norm = f32::from_le_bytes(v_chunk[offset + 4..offset + 8].try_into().unwrap());
            let z_norm = f32::from_le_bytes(v_chunk[offset + 8..offset + 12].try_into().unwrap());
            normals.push([x_norm, y_norm, z_norm]); // 同样翻转Z轴
            offset += 12;

            // --- 修正点 2: UV坐标是两个f32 (8字节)，不是两个f16 (4字节) ---
            let u = f32::from_le_bytes(v_chunk[offset..offset + 4].try_into().unwrap());
            let v = f32::from_le_bytes(v_chunk[offset + 4..offset + 8].try_into().unwrap());
            uvs.push([u, v]); // UV的V坐标通常也需要翻转 (从上到下 -> 从下到上)
            offset += 8;

            // 解析可选属性
            if let Some(colors_vec) = colors.as_mut() {
                // 顶点颜色 (4 bytes, u8 -> f32 normalized)
                let color_u8: [u8; 4] = v_chunk[offset..offset + 4].try_into().unwrap();
                colors_vec.push([
                    // BGRA -> RGBA 转换并归一化
                    color_u8[2] as f32 / 255.0,
                    color_u8[1] as f32 / 255.0,
                    color_u8[0] as f32 / 255.0,
                    color_u8[3] as f32 / 255.0,
                ]);
                offset += 4;
            }

            if let Some(tangents_vec) = tangents.as_mut() {
                // 切线 (16 bytes)
                let tan_x = f32::from_le_bytes(v_chunk[offset..offset + 4].try_into().unwrap());
                let tan_y = f32::from_le_bytes(v_chunk[offset + 4..offset + 8].try_into().unwrap());
                let tan_z =
                    f32::from_le_bytes(v_chunk[offset + 8..offset + 12].try_into().unwrap());
                let tan_w =
                    f32::from_le_bytes(v_chunk[offset + 12..offset + 16].try_into().unwrap());
                // 同样翻转Z轴
                tangents_vec.push([tan_x, tan_y, tan_z, tan_w]);
                // offset += 16; // 已是最后一个元素，无需增加offset
            }
        }

        // 6. 获取、解析并调整索引
        let index_start_byte = range.start_index as usize * 2; // u16 = 2 bytes
        let index_end_byte = index_start_byte + (range.index_count as usize * 2);
        let index_data_slice = self.index_buffer.get(index_start_byte..index_end_byte)?;

        let local_indices: Vec<u16> = index_data_slice
            .chunks_exact(2)
            .map(|bytes| u16::from_le_bytes(bytes.try_into().unwrap()))
            .map(|global_index| global_index - range.start_vertex as u16) // 将全局索引转换为局部索引
            .collect();

        // 7. 修正因Z轴翻转导致的三角形环绕顺序问题
        // for tri_indices in local_indices.chunks_exact_mut(3) {
        //     tri_indices.swap(1, 2); // 从 [0, 1, 2] -> [0, 2, 1]
        // }

        // 8. 创建 Bevy Mesh 并插入所有顶点属性
        let mut bevy_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        // --- 修正点 3: 启用蒙皮和颜色属性 ---
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

        // 9. 设置网格索引
        bevy_mesh.insert_indices(Indices::U16(local_indices));

        Some(bevy_mesh)
    }
}

/// (内部使用) 用于直接从二进制文件解析的结构体。
/// 它包含了所有复杂的条件解析逻辑。
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

    // --- 临时解析字段 ---
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

    // --- 计算字段 ---
    #[br(calc = index_count_v0.or(index_count_v2_4).unwrap_or(0))]
    pub index_count: u32,
    #[br(calc = vertex_count_v0.or(vertex_count_v2_4).unwrap_or(0))]
    pub vertex_count: u32,
    #[br(calc = Self::parse_vertex_declaration(major, vertex_size, vertex_type_raw))]
    pub vertex_declaration: SkinnedMeshVertex,

    // --- 数据缓冲区 ---
    #[br(count = index_count * 2)]
    pub index_buffer: Vec<u8>,
    #[br(count = vertex_count * vertex_declaration.get_vertex_size())]
    pub vertex_buffer: Vec<u8>,
}

impl LeagueSkinnedMeshInternal {
    // 辅助函数保持不变
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

/// 实现 `From` Trait 以进行后处理和转换
impl From<LeagueSkinnedMeshInternal> for LeagueSkinnedMesh {
    fn from(internal: LeagueSkinnedMeshInternal) -> Self {
        let mut final_ranges = internal.ranges;
        // 在这里进行后处理，而不是在 map 函数中
        if internal.major == 0 {
            final_ranges = vec![SkinnedMeshRange {
                name: "".to_string(),
                start_vertex: 0,
                vertex_count: internal.vertex_count,
                start_index: 0,
                index_count: internal.index_count,
            }];
        }

        // 将内部结构体的字段转移到公开结构体
        Self {
            major: internal.major,
            minor: internal.minor,
            ranges: final_ranges, // 使用处理过的 ranges
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
