use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use binrw::{BinRead, BinWrite};

/// 序列化字符串结构
#[derive(Debug, Clone, BinRead, BinWrite)]
#[br(little)]
#[bw(little)]
pub struct SerializableString {
    length: u32,
    #[br(count = length)]
    data: Vec<u8>,
}

impl From<String> for SerializableString {
    fn from(s: String) -> Self {
        let data = s.into_bytes();
        Self {
            length: data.len() as u32,
            data,
        }
    }
}

impl From<SerializableString> for String {
    fn from(s: SerializableString) -> Self {
        String::from_utf8_lossy(&s.data).to_string()
    }
}

impl From<&str> for SerializableString {
    fn from(s: &str) -> Self {
        let data = s.as_bytes().to_vec();
        Self {
            length: data.len() as u32,
            data,
        }
    }
}

/// 中间mesh结构，用于高效序列化和反序列化
/// 支持从League mesh格式转换，也支持转换为Bevy mesh
#[derive(Debug, Clone, BinRead, BinWrite)]
#[br(little)]
#[bw(little)]
pub struct IntermediateMesh {
    /// mesh名称或标识
    pub name: SerializableString,

    /// 顶点数量，用于验证其他数据长度
    pub vertex_count: u32,

    /// 顶点位置数据
    #[br(count = vertex_count)]
    pub positions: Vec<[f32; 3]>,

    /// 法线数据（可选）
    pub has_normals: u8,
    #[br(if(has_normals != 0), count = vertex_count)]
    pub normals: Option<Vec<[f32; 3]>>,

    /// UV坐标数据（可选）
    pub has_uvs: u8,
    #[br(if(has_uvs != 0), count = vertex_count)]
    pub uvs: Option<Vec<[f32; 2]>>,

    /// 顶点颜色数据（可选）
    pub has_colors: u8,
    #[br(if(has_colors != 0), count = vertex_count)]
    pub colors: Option<Vec<[f32; 4]>>,

    /// 切线数据（可选）
    pub has_tangents: u8,
    #[br(if(has_tangents != 0), count = vertex_count)]
    pub tangents: Option<Vec<[f32; 4]>>,

    /// 骨骼索引数据（可选，用于骨骼动画）
    pub has_joint_indices: u8,
    #[br(if(has_joint_indices != 0), count = vertex_count)]
    pub joint_indices: Option<Vec<[u16; 4]>>,

    /// 骨骼权重数据（可选，用于骨骼动画）
    pub has_joint_weights: u8,
    #[br(if(has_joint_weights != 0), count = vertex_count)]
    pub joint_weights: Option<Vec<[f32; 4]>>,

    /// 索引数量
    pub index_count: u32,
    /// 索引数据
    #[br(count = index_count)]
    pub indices: Vec<u16>,

    /// 材质信息（可选）
    pub has_material_info: u8,
    #[br(if(has_material_info != 0))]
    pub material_info: Option<SerializableString>,
}

impl IntermediateMesh {
    /// 创建一个新的空mesh
    pub fn new(name: String) -> Self {
        Self {
            name: name.into(),
            vertex_count: 0,
            positions: Vec::new(),
            has_normals: 0,
            normals: None,
            has_uvs: 0,
            uvs: None,
            has_colors: 0,
            colors: None,
            has_tangents: 0,
            tangents: None,
            has_joint_indices: 0,
            joint_indices: None,
            has_joint_weights: 0,
            joint_weights: None,
            index_count: 0,
            indices: Vec::new(),
            has_material_info: 0,
            material_info: None,
        }
    }

    /// 检查mesh是否包含骨骼动画数据
    pub fn is_skinned(&self) -> bool {
        self.joint_indices.is_some() && self.joint_weights.is_some()
    }

    /// 获取顶点数量
    pub fn vertex_count(&self) -> usize {
        self.vertex_count as usize
    }

    /// 获取三角形数量
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// 设置顶点位置（自动更新vertex_count）
    pub fn set_positions(&mut self, positions: Vec<[f32; 3]>) {
        self.vertex_count = positions.len() as u32;
        self.positions = positions;
    }

    /// 设置索引数据（自动更新index_count）
    pub fn set_indices(&mut self, indices: Vec<u16>) {
        self.index_count = indices.len() as u32;
        self.indices = indices;
    }

    /// 设置法线数据
    pub fn set_normals(&mut self, normals: Option<Vec<[f32; 3]>>) {
        self.has_normals = if normals.is_some() { 1 } else { 0 };
        self.normals = normals;
    }

    /// 设置UV数据
    pub fn set_uvs(&mut self, uvs: Option<Vec<[f32; 2]>>) {
        self.has_uvs = if uvs.is_some() { 1 } else { 0 };
        self.uvs = uvs;
    }

    /// 设置颜色数据
    pub fn set_colors(&mut self, colors: Option<Vec<[f32; 4]>>) {
        self.has_colors = if colors.is_some() { 1 } else { 0 };
        self.colors = colors;
    }

    /// 设置切线数据
    pub fn set_tangents(&mut self, tangents: Option<Vec<[f32; 4]>>) {
        self.has_tangents = if tangents.is_some() { 1 } else { 0 };
        self.tangents = tangents;
    }

    /// 设置骨骼索引数据
    pub fn set_joint_indices(&mut self, joint_indices: Option<Vec<[u16; 4]>>) {
        self.has_joint_indices = if joint_indices.is_some() { 1 } else { 0 };
        self.joint_indices = joint_indices;
    }

    /// 设置骨骼权重数据
    pub fn set_joint_weights(&mut self, joint_weights: Option<Vec<[f32; 4]>>) {
        self.has_joint_weights = if joint_weights.is_some() { 1 } else { 0 };
        self.joint_weights = joint_weights;
    }

    /// 设置材质信息
    pub fn set_material_info(&mut self, material_info: Option<String>) {
        self.has_material_info = if material_info.is_some() { 1 } else { 0 };
        self.material_info = material_info.map(|s| s.into());
    }

    /// 转换为Bevy mesh
    /// 这个方法优化了性能，避免不必要的数据复制
    pub fn to_bevy_mesh(&self) -> Mesh {
        let mut bevy_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        // 插入必需的位置属性
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions.clone());

        // 插入可选属性
        if let Some(ref normals) = self.normals {
            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
        }

        if let Some(ref uvs) = self.uvs {
            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone());
        }

        if let Some(ref colors) = self.colors {
            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors.clone());
        }

        if let Some(ref tangents) = self.tangents {
            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents.clone());
        }

        // 插入骨骼动画属性
        if let Some(ref joint_indices) = self.joint_indices {
            bevy_mesh.insert_attribute(
                Mesh::ATTRIBUTE_JOINT_INDEX,
                VertexAttributeValues::Uint16x4(joint_indices.clone()),
            );
        }

        if let Some(ref joint_weights) = self.joint_weights {
            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, joint_weights.clone());
        }

        // 插入索引
        bevy_mesh.insert_indices(Indices::U16(self.indices.clone()));

        bevy_mesh
    }

    /// 验证mesh数据的完整性
    pub fn validate(&self) -> Result<(), String> {
        if self.vertex_count == 0 {
            return Err("Mesh must have at least one vertex".to_string());
        }

        let vertex_count = self.vertex_count as usize;

        // 检查positions长度
        if self.positions.len() != vertex_count {
            return Err("Positions count doesn't match vertex_count".to_string());
        }

        // 检查所有顶点属性的长度是否一致
        if let Some(ref normals) = self.normals {
            if normals.len() != vertex_count {
                return Err("Normals count doesn't match vertex count".to_string());
            }
        }

        if let Some(ref uvs) = self.uvs {
            if uvs.len() != vertex_count {
                return Err("UVs count doesn't match vertex count".to_string());
            }
        }

        if let Some(ref colors) = self.colors {
            if colors.len() != vertex_count {
                return Err("Colors count doesn't match vertex count".to_string());
            }
        }

        if let Some(ref tangents) = self.tangents {
            if tangents.len() != vertex_count {
                return Err("Tangents count doesn't match vertex count".to_string());
            }
        }

        if let Some(ref joint_indices) = self.joint_indices {
            if joint_indices.len() != vertex_count {
                return Err("Joint indices count doesn't match vertex count".to_string());
            }
        }

        if let Some(ref joint_weights) = self.joint_weights {
            if joint_weights.len() != vertex_count {
                return Err("Joint weights count doesn't match vertex count".to_string());
            }
        }

        // 检查索引长度
        if self.indices.len() != self.index_count as usize {
            return Err("Indices length doesn't match index_count".to_string());
        }

        // 检查索引是否有效
        for &index in &self.indices {
            if index as usize >= vertex_count {
                return Err(format!(
                    "Index {} is out of bounds for {} vertices",
                    index, vertex_count
                ));
            }
        }

        // 检查索引数量是否是3的倍数
        if self.indices.len() % 3 != 0 {
            return Err("Index count must be a multiple of 3 for triangle lists".to_string());
        }

        Ok(())
    }
}
