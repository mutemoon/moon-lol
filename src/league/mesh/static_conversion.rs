use super::intermediate::IntermediateMesh;
use crate::league::{ElementName, LeagueMapGeo, LeagueMapGeoMesh, Submesh};
use std::collections::HashMap;

/// 从静态mesh（submesh）创建中间结构
pub fn submesh_to_intermediate(
    submesh: &Submesh,
    map_file: &LeagueMapGeo,
    map_mesh: &LeagueMapGeoMesh,
    all_positions: &Vec<[f32; 3]>,
    all_normals: &Vec<[f32; 3]>,
    all_uvs: &Vec<[f32; 2]>,
) -> Option<IntermediateMesh> {
    // 获取索引数据
    let index_buffer = map_file
        .index_buffers
        .get(map_mesh.index_buffer_id as usize)?;
    let all_indices = &index_buffer.buffer;

    // 获取当前submesh的索引范围
    let start = submesh.start_index as usize;
    let end = start + submesh.submesh_index_count as usize;
    if end > all_indices.len() {
        return None;
    }
    let global_indices_slice = &all_indices[start..end];

    // 创建局部顶点数据和索引映射
    let mut local_positions = Vec::new();
    let mut local_normals = Vec::new();
    let mut local_uvs = Vec::new();
    let mut local_indices = Vec::with_capacity(global_indices_slice.len());
    let mut global_to_local_map = HashMap::new();

    // 重新映射顶点数据，只保留当前submesh使用的顶点
    for &global_index in global_indices_slice {
        let local_index = *global_to_local_map.entry(global_index).or_insert_with(|| {
            let new_local_index = local_positions.len() as u16;

            // 添加位置数据
            if let Some(pos) = all_positions.get(global_index as usize) {
                local_positions.push(*pos);
            }

            // 添加法线数据
            if let Some(normal) = all_normals.get(global_index as usize) {
                local_normals.push(*normal);
            }

            // 添加UV数据
            if let Some(uv) = all_uvs.get(global_index as usize) {
                local_uvs.push(*uv);
            }

            new_local_index
        });
        local_indices.push(local_index);
    }

    // 修正三角形绕序（与原始代码保持一致）
    for tri_indices in local_indices.chunks_exact_mut(3) {
        tri_indices.swap(1, 2);
    }

    // 创建中间mesh结构
    let mut intermediate_mesh = IntermediateMesh::new(submesh.material_name.text.clone());

    intermediate_mesh.set_positions(local_positions);

    // 只有当数据不为空时才设置可选属性
    if !local_normals.is_empty() {
        intermediate_mesh.set_normals(Some(local_normals));
    }

    if !local_uvs.is_empty() {
        intermediate_mesh.set_uvs(Some(local_uvs));
    }

    intermediate_mesh.set_indices(local_indices);
    intermediate_mesh.set_material_info(Some(submesh.material_name.text.clone()));

    Some(intermediate_mesh)
}

/// 从 MapGeoMesh 中解析出所有顶点属性，作为共享的全局数据池。
/// 返回一个元组，包含所有顶点的位置、法线和 UV 坐标。
pub fn parse_vertex_data(
    map_file: &LeagueMapGeo,
    map_mesh: &LeagueMapGeoMesh,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>) {
    // 预分配容量可以轻微提升性能，但需要估算大小。为简化，这里省略。
    let mut all_positions = Vec::new();
    let mut all_normals = Vec::new();
    let mut all_uvs = Vec::new();

    for v_decl_idx_offset in 0..map_mesh.vertex_declaration_count as usize {
        let decl_index = (map_mesh.vertex_declaration_index_base as usize) + v_decl_idx_offset;
        let v_buff_index = map_mesh.vertex_buffer_indexes[v_decl_idx_offset] as usize;

        let declaration = &map_file.vertex_declarations[decl_index];
        let vertex_buffer = &map_file.vertex_buffers[v_buff_index];
        let buffer_data = &vertex_buffer.buffer;

        // 计算顶点声明的总步长（单个顶点占用的字节数）
        let stride = declaration
            .elements
            .iter()
            .map(|e| e.format.get_size())
            .sum::<usize>();

        if stride == 0 {
            continue;
        }

        // 遍历顶点缓冲区中的每一个顶点
        for vtx_chunk in buffer_data.chunks_exact(stride) {
            let mut offset = 0;
            // 遍历顶点声明中的每一个元素（如位置、法线等）
            for element in &declaration.elements {
                let size = element.format.get_size();
                let element_data = &vtx_chunk[offset..offset + size];

                match element.name {
                    ElementName::Position => {
                        if element_data.len() >= 12 {
                            let x = f32::from_le_bytes(element_data[0..4].try_into().unwrap());
                            let y = f32::from_le_bytes(element_data[4..8].try_into().unwrap());
                            let z = f32::from_le_bytes(element_data[8..12].try_into().unwrap());
                            // Bevy 使用右手坐标系 (Y-up)，这里根据需要翻转Z轴
                            all_positions.push([x, y, -z]);
                        }
                    }
                    ElementName::Normal => {
                        if element_data.len() >= 12 {
                            let x = f32::from_le_bytes(element_data[0..4].try_into().unwrap());
                            let y = f32::from_le_bytes(element_data[4..8].try_into().unwrap());
                            let z = f32::from_le_bytes(element_data[8..12].try_into().unwrap());
                            // 同样翻转Z轴以匹配坐标系
                            all_normals.push([x, y, -z]);
                        }
                    }
                    ElementName::Texcoord0 => {
                        if element_data.len() >= 8 {
                            let u = f32::from_le_bytes(element_data[0..4].try_into().unwrap());
                            let v = f32::from_le_bytes(element_data[4..8].try_into().unwrap());
                            all_uvs.push([u, v]);
                        }
                    }
                    // 根据需要可以添加对其他元素（如颜色、骨骼权重等）的解析
                    _ => {}
                }
                offset += size;
            }
        }
    }

    (all_positions, all_normals, all_uvs)
}
