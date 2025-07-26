use crate::render::{BoundingBox, EnvironmentVisibility, LeagueLoader, SizedString, Vector2};
use crate::render::{ElementName, LeagueMapGeo};
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use binrw::binread;
use cdragon_prop::{BinEmbed, BinEntry, BinList, BinString, PropFile};
use std::collections::HashMap;

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct LeagueMapGeoMesh {
    pub vertex_count: u32,
    pub vertex_declaration_count: u32,
    pub vertex_declaration_index_base: u32,

    #[br(count = vertex_declaration_count)]
    pub vertex_buffer_indexes: Vec<u32>,

    pub index_count: u32,
    pub index_buffer_id: u32,

    pub environment_visibility: EnvironmentVisibility,
    pub visibility_controller_path_hash: u32,

    pub submesh_count: u32,
    #[br(count = submesh_count)]
    pub submeshes: Vec<Submesh>,

    #[br(map = |v: u8| v != 0)]
    pub disable_backface_culling: bool,

    pub bounding_box: BoundingBox,

    #[br(count = 16)]
    pub transform: Vec<f32>,

    #[br(map = |v: u8| parse_quality_filter(v))]
    pub quality_filter: QualityFilter,

    #[br(map = |v: u8| parse_layer_transition_behavior(v))]
    pub layer_transition_behavior: LayerTransitionBehavior,

    pub render_flags: u16,

    pub baked_light: Channel,
    pub stationary_light: Channel,

    pub texture_override_count: u32,
    #[br(count = texture_override_count)]
    pub texture_overrides: Vec<TextureOverride>,

    pub baked_paint_scale: Vector2,
    pub baked_paint_bias: Vector2,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct Submesh {
    pub hash: u32,

    pub material_name: SizedString,

    pub start_index: u32,
    pub submesh_index_count: u32,
    pub min_vertex: u32,
    pub max_vertex: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum QualityFilter {
    All,
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

fn parse_quality_filter(val: u8) -> QualityFilter {
    match val {
        1 => QualityFilter::VeryLow,
        2 => QualityFilter::Low,
        4 => QualityFilter::Medium,
        8 => QualityFilter::High,
        16 => QualityFilter::VeryHigh,
        _ => QualityFilter::All,
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LayerTransitionBehavior {
    Unaffected,
    TurnInvisibleDoesNotMatchNewLayerFilter,
    TurnVisibleDoesMatchNewLayerFilter,
}

fn parse_layer_transition_behavior(val: u8) -> LayerTransitionBehavior {
    match val {
        1 => LayerTransitionBehavior::TurnInvisibleDoesNotMatchNewLayerFilter,
        2 => LayerTransitionBehavior::TurnVisibleDoesMatchNewLayerFilter,
        _ => LayerTransitionBehavior::Unaffected,
    }
}

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct Channel {
    pub texture: SizedString,
    pub uv_scale: Vector2,
    pub uv_offset: Vector2,
}

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct TextureOverride {
    pub sampler_id: u32,
    pub texture_path: SizedString,
}

// pub fn process_map_geo_mesh(
//     map_materials: &PropFile,
//     map_file: &LeagueMapGeo,
//     map_mesh: &LeagueMapGeoMesh,
//     league_loader: &LeagueLoader,
// ) -> Vec<(Mesh, Image)> {
//     // ---- 1. 为整个 MapGeoMesh 提取所有顶点数据（全局缓冲） ----
//     let mut all_positions: Vec<[f32; 3]> = Vec::new();
//     let mut all_normals: Vec<[f32; 3]> = Vec::new();
//     let mut all_uvs: Vec<[f32; 2]> = Vec::new();

//     // 这部分逻辑与你原有的代码几乎一致，用于填充上述全局缓冲
//     for v_decl_idx_offset in 0..map_mesh.vertex_declaration_count as usize {
//         let decl_index = (map_mesh.vertex_declaration_index_base as usize) + v_decl_idx_offset;
//         let v_buff_index = map_mesh.vertex_buffer_indexes[v_decl_idx_offset] as usize;

//         let declaration = &map_file.vertex_declarations[decl_index];
//         let vertex_buffer = &map_file.vertex_buffers[v_buff_index];
//         let buffer_data = &vertex_buffer.buffer;
//         let stride = declaration
//             .elements
//             .iter()
//             .map(|e| e.format.get_size())
//             .sum::<usize>();

//         if stride == 0 {
//             continue;
//         }

//         for vtx_chunk in buffer_data.chunks_exact(stride) {
//             let mut offset = 0;
//             for element in &declaration.elements {
//                 let size = element.format.get_size();
//                 let element_data = &vtx_chunk[offset..offset + size];

//                 // 为了简洁，这里仅展示解析逻辑，实际应使用你的完整代码
//                 match element.name {
//                     ElementName::Position => {
//                         if element_data.len() >= 12 {
//                             let x = f32::from_le_bytes([
//                                 element_data[0],
//                                 element_data[1],
//                                 element_data[2],
//                                 element_data[3],
//                             ]);
//                             let y = f32::from_le_bytes([
//                                 element_data[4],
//                                 element_data[5],
//                                 element_data[6],
//                                 element_data[7],
//                             ]);
//                             let z = f32::from_le_bytes([
//                                 element_data[8],
//                                 element_data[9],
//                                 element_data[10],
//                                 element_data[11],
//                             ]);
//                             all_positions.push([x, y, -z]);
//                         }
//                     }
//                     ElementName::Normal => {
//                         if element_data.len() >= 12 {
//                             let x = f32::from_le_bytes([
//                                 element_data[0],
//                                 element_data[1],
//                                 element_data[2],
//                                 element_data[3],
//                             ]);
//                             let y = f32::from_le_bytes([
//                                 element_data[4],
//                                 element_data[5],
//                                 element_data[6],
//                                 element_data[7],
//                             ]);
//                             let z = f32::from_le_bytes([
//                                 element_data[8],
//                                 element_data[9],
//                                 element_data[10],
//                                 element_data[11],
//                             ]);
//                             all_normals.push([x, y, -z]);
//                         }
//                     }
//                     ElementName::Texcoord0 => {
//                         if element_data.len() >= 8 {
//                             let u = f32::from_le_bytes([
//                                 element_data[0],
//                                 element_data[1],
//                                 element_data[2],
//                                 element_data[3],
//                             ]);
//                             let v = f32::from_le_bytes([
//                                 element_data[4],
//                                 element_data[5],
//                                 element_data[6],
//                                 element_data[7],
//                             ]);
//                             all_uvs.push([u, v]);
//                         }
//                     }
//                     _ => {}
//                 }
//                 offset += size;
//             }
//         }
//     }
//     // (请在此处填入你完整的顶点解析逻辑来填充 all_positions, all_normals, all_uvs)

//     // ---- 2. 获取完整的索引缓冲区 ----
//     let index_buffer = &map_file.index_buffers[map_mesh.index_buffer_id as usize];
//     let all_indices = &index_buffer.buffer;

//     // ---- 3. 遍历每个 Submesh 并创建独立的 Mesh ----
//     let mut result_bundles: Vec<_> = Vec::new();

//     for submesh in &map_mesh.submeshes {
//         // a. 获取此 submesh 对应的索引切片
//         let start = submesh.start_index as usize;
//         let end = start + submesh.submesh_index_count as usize;
//         if end > all_indices.len() {
//             eprintln!("Submesh index range is out of bounds. Skipping.");
//             continue;
//         }
//         let global_indices_slice = &all_indices[start..end];

//         // b. 为此 submesh 创建局部的顶点和索引数据
//         let mut local_positions: Vec<[f32; 3]> = Vec::new();
//         let mut local_normals: Vec<[f32; 3]> = Vec::new();
//         let mut local_uvs: Vec<[f32; 2]> = Vec::new();
//         let mut local_indices: Vec<u16> = Vec::with_capacity(global_indices_slice.len());

//         // `global_to_local_map` 用于重映射索引
//         let mut global_to_local_map: HashMap<u16, u16> = HashMap::new();

//         for &global_index in global_indices_slice {
//             let local_index = *global_to_local_map.entry(global_index).or_insert_with(|| {
//                 // 如果是新顶点，则从全局缓冲中复制其属性
//                 let new_local_index = local_positions.len() as u16;

//                 if let Some(pos) = all_positions.get(global_index as usize) {
//                     local_positions.push(*pos);
//                 }
//                 if let Some(normal) = all_normals.get(global_index as usize) {
//                     local_normals.push(*normal);
//                 }
//                 if let Some(uv) = all_uvs.get(global_index as usize) {
//                     local_uvs.push(*uv);
//                 }

//                 new_local_index
//             });
//             local_indices.push(local_index);
//         }

//         // c. 修正因Z轴翻转导致的三角形环绕顺序问题
//         for tri_indices in local_indices.chunks_exact_mut(3) {
//             tri_indices.swap(1, 2);
//         }

//         // d. 创建 Bevy Mesh
//         let mut bevy_mesh = Mesh::new(
//             PrimitiveTopology::TriangleList,
//             RenderAssetUsages::default(),
//         );

//         bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, local_positions);
//         if !local_normals.is_empty() {
//             bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, local_normals);
//         }
//         if !local_uvs.is_empty() {
//             bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, local_uvs);
//         }

//         bevy_mesh.insert_indices(Indices::U16(local_indices));

//         // 如果法线数据不存在，可以尝试计算（但你的格式似乎总是有法线）
//         if bevy_mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_none() {
//             let _ = bevy_mesh.compute_flat_normals();
//         }

//         let binhash = LeagueLoader::compute_binhash(&submesh.material_name.text);

//         let mut bevy_image = None;
//         for entry in &map_materials.entries {
//             if entry.path.hash == binhash {
//                 let Some(sampler_values) =
//                     entry.getv::<BinList>(LeagueLoader::compute_binhash("samplerValues").into())
//                 else {
//                     continue;
//                 };
//                 let Some(res) = sampler_values.downcast::<BinEmbed>() else {
//                     continue;
//                 };
//                 let res: Vec<_> = res
//                     .into_iter()
//                     .filter_map(|v| {
//                         v.getv::<BinString>(LeagueLoader::compute_binhash("texturePath").into())
//                     })
//                     .map(|v| v.0.clone())
//                     .filter_map(|texture_path| {
//                         println!("Loading texture: {}", texture_path);
//                         match league_loader.get_image_by_texture_path(&texture_path) {
//                             Ok(image) => Some(image),
//                             Err(e) => {
//                                 warn!("Failed to load texture {}: {}", texture_path, e);
//                                 None
//                             }
//                         }
//                     })
//                     .collect();
//                 bevy_image = res.first().cloned();
//             }
//         }

//         // e. 创建并存储 SubmeshBundle
//         let fallback_image = if bevy_image.is_none() {
//             // Create a simple fallback texture (1x1 white pixel)
//             let mut fallback = Image::new_fill(
//                 Extent3d {
//                     width: 1,
//                     height: 1,
//                     depth_or_array_layers: 1,
//                 },
//                 TextureDimension::D2,
//                 &[255, 255, 255, 255], // White RGBA
//                 TextureFormat::Rgba8Unorm,
//                 RenderAssetUsages::default(),
//             );
//             fallback.sampler = ImageSampler::linear();
//             Some(fallback)
//         } else {
//             bevy_image
//         };

//         let bundle = (bevy_mesh, fallback_image.unwrap());
//         result_bundles.push(bundle);
//     }

//     result_bundles
// }

/// 从 MapGeoMesh 中解析出所有顶点属性，作为共享的全局数据池。
/// 返回一个元组，包含所有顶点的位置、法线和 UV 坐标。
fn parse_vertex_data(
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

// ---- 辅助函数 2：为 Submesh 创建独立的 Bevy Mesh ----
/// 根据单个 submesh 的索引范围，从全局顶点数据中提取数据，
/// 创建一个独立的、自包含的 Bevy Mesh。
fn create_bevy_mesh_for_submesh(
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

// ---- 辅助函数 3：查找并加载贴图 ----
/// 根据 submesh 的材质名，查找材质属性并加载对应的贴图文件。
/// 如果失败，则返回一个默认的白色贴图。
fn find_and_load_image_for_submesh(
    submesh: &Submesh,
    map_materials: &PropFile,
    league_loader: &LeagueLoader,
) -> Image {
    // 1. 根据材质名查找 texturePath
    let binhash = LeagueLoader::compute_binhash(&submesh.material_name.text);
    for entry in &map_materials.entries {
        if entry.path.hash == binhash {
            // ... 你原来的材质查找逻辑 ...
            // 假设你找到了 texture_path: String
            if let Some(texture_path) = find_texture_path_in_material_entry(entry) {
                match league_loader.get_image_by_texture_path(&texture_path) {
                    Ok(image) => return image,
                    Err(e) => warn!("Failed to load texture {}: {}", texture_path, e),
                }
            }
        }
    }

    // 2. 如果找不到或加载失败，返回一个备用贴图
    let mut fallback = Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255, 255, 255, 255], // White RGBA
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    fallback.sampler = ImageSampler::linear();
    fallback
}

/// 在单个材质条目中查找 "texturePath" 的值。
/// 材质属性通常是嵌套的，结构为：samplerValues -> (list) -> (embed) -> texturePath。
/// 这里使用了 `find_map` 来高效地找到第一个匹配项。
fn find_texture_path_in_material_entry(
    material_entry: &BinEntry, // 请替换为你的材质条目具体类型
) -> Option<String> {
    // 1. 获取 "samplerValues" 列表
    let sampler_values =
        material_entry.getv::<BinList>(LeagueLoader::compute_binhash("samplerValues").into())?;

    // 2. 将列表转换为可迭代的 BinEmbed
    let embedded_samplers = sampler_values.downcast::<BinEmbed>()?;

    // 3. 遍历所有 sampler，查找第一个包含 "texturePath" 的
    // `find_map` 会在找到第一个 Some(T) 后立即停止，比 filter_map + collect + first 更高效
    embedded_samplers.into_iter().find_map(|sampler_item| {
        let texture_name = &sampler_item
            .getv::<BinString>(LeagueLoader::compute_binhash("textureName").into())?
            .0;
        if texture_name != "DiffuseTexture" {
            return None;
        }
        sampler_item
            .getv::<BinString>(LeagueLoader::compute_binhash("texturePath").into())
            .map(|v| v.0.clone())
    })
}

// ---- 主协调函数 ----
/// 处理单个 MapGeoMesh，并行地为其所有 submesh 创建 Bevy Mesh 和 Image。
pub fn process_map_geo_mesh(
    map_materials: &PropFile,
    map_file: &LeagueMapGeo,
    map_mesh: &LeagueMapGeoMesh,
    league_loader: &LeagueLoader,
) -> Vec<(Mesh, Image)> {
    // 步骤 1: 一次性解析所有顶点数据
    let (all_positions, all_normals, all_uvs) = parse_vertex_data(map_file, map_mesh);

    // 步骤 2: 一次性获取索引缓冲区的引用
    let index_buffer = &map_file.index_buffers[map_mesh.index_buffer_id as usize];
    let all_indices = &index_buffer.buffer;

    // 步骤 3: 并行处理所有 submesh
    let result_bundles: Vec<(Mesh, Image)> = map_mesh
        .submeshes
        .iter()
        .filter_map(|submesh| {
            // 获取当前 submesh 对应的索引切片
            let start = submesh.start_index as usize;
            let end = start + submesh.submesh_index_count as usize;
            if end > all_indices.len() {
                return None;
            }
            let global_indices_slice = &all_indices[start..end];

            // 为 submesh 创建独立的 Mesh
            let bevy_mesh = create_bevy_mesh_for_submesh(
                global_indices_slice,
                &all_positions,
                &all_normals,
                &all_uvs,
            );

            // 为 submesh 查找并加载贴图
            let bevy_image = find_and_load_image_for_submesh(submesh, map_materials, league_loader);

            Some((bevy_mesh, bevy_image))
        })
        .collect();

    result_bundles
}
