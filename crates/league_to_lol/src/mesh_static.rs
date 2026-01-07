use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh};
use bevy::render::render_resource::PrimitiveTopology;
use league_file::LeagueMeshStatic;

pub fn mesh_static_to_bevy_mesh(mesh: LeagueMeshStatic) -> Mesh {
    let num_vertices = mesh.faces.len() * 3;

    // 1. 准备好展开后的顶点属性 Vec
    let mut bevy_positions: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
    let mut bevy_uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);

    // 只有当源 mesh 包含顶点色时才创建 Vec
    let mut bevy_colors: Option<Vec<[f32; 4]>> = if mesh.has_vertex_colors {
        Some(Vec::with_capacity(num_vertices))
    } else {
        None
    };

    // 2. 遍历所有面，展开顶点数据
    for face in &mesh.faces {
        for i in 0..3 {
            // 获取面中第 i 个顶点的索引，指向全局 "vertices" 列表
            let global_pos_index = face.indices[i] as usize;

            // a. 获取位置数据
            // 从全局 "vertices" 列表查找位置
            let pos = mesh.vertices[global_pos_index];
            bevy_positions.push(pos);

            // c. 获取UV数据
            // UV 数据也直接存储在 face 中
            let uv = face.uvs[i];
            bevy_uvs.push(uv);

            // d. 获取顶点色数据（如果存在）
            if let Some(colors_vec) = bevy_colors.as_mut() {
                // 顶点色和位置一样，从全局列表 "vertex_colors" 中查找
                let bgra_u8 = mesh.vertex_colors.as_ref().unwrap()[global_pos_index];

                // 转换: [u8; 4] (BGRA) -> [f32; 4] (RGBA, normalized)
                // 参考 skin_mesh.rs 的转换
                colors_vec.push([
                    bgra_u8[2] as f32 / 255.0, // R
                    bgra_u8[1] as f32 / 255.0, // G
                    bgra_u8[0] as f32 / 255.0, // B
                    bgra_u8[3] as f32 / 255.0, // A
                ]);
            }
        }
    }

    // 3. 创建索引
    // 因为我们展开了所有顶点，索引现在只是一个简单的 0, 1, 2, 3, 4, 5, ... 序列
    let indices: Vec<u16> = (0..num_vertices as u16).collect();

    // 4. 创建 Bevy Mesh 并插入所有属性
    let mut bevy_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, bevy_positions);
    bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, bevy_uvs);

    if let Some(colors_data) = bevy_colors {
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors_data);
    }

    bevy_mesh.insert_indices(Indices::U16(indices));

    bevy_mesh.compute_normals();

    bevy_mesh
}
