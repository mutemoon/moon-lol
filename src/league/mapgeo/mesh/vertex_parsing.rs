use crate::league::mapgeo::element_name::ElementName;
use crate::league::mapgeo::mesh::static_mesh::LeagueMapGeoMesh;
use crate::league::LeagueMapGeo;

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
