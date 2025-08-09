use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use std::collections::HashMap;

/// 根据单个 submesh 的索引范围，从全局顶点数据中提取数据，
/// 创建一个独立的、自包含的 Bevy Mesh。
pub fn create_bevy_mesh_for_submesh(
    global_indices_slice: &[u16],
    all_positions: &[[f32; 3]],
    all_normals: &[[f32; 3]],
    all_uvs: &[[f32; 2]],
) -> Mesh {
    let mut local_positions = Vec::new();
    let mut local_normals = Vec::new();
    let mut local_uvs = Vec::new();
    let mut local_indices = Vec::with_capacity(global_indices_slice.len());
    let mut global_to_local_map = HashMap::new();

    for &global_index in global_indices_slice {
        let local_index = *global_to_local_map.entry(global_index).or_insert_with(|| {
            let new_local_index = local_positions.len() as u16;
            if let Some(pos) = all_positions.get(global_index as usize) {
                local_positions.push(*pos);
            }
            if let Some(normal) = all_normals.get(global_index as usize) {
                local_normals.push(*normal);
            }
            if let Some(uv) = all_uvs.get(global_index as usize) {
                local_uvs.push(*uv);
            }
            new_local_index
        });
        local_indices.push(local_index);
    }

    // 修正因Z轴翻转导致的三角形环绕顺序问题
    for tri_indices in local_indices.chunks_exact_mut(3) {
        tri_indices.swap(1, 2);
    }

    let mut bevy_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, local_positions);
    if !local_normals.is_empty() {
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, local_normals);
    }
    if !local_uvs.is_empty() {
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, local_uvs);
    }
    bevy_mesh.insert_indices(Indices::U16(local_indices));

    bevy_mesh
}
