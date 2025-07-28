use crate::render::{BoundingBox, EnvironmentVisibility, LeagueLoader, SizedString, Vector2};
use crate::render::{ElementName, LeagueMapGeo};
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
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
    let is_debug = submesh.material_name.text
        == "Maps/KitPieces/SRX/Chemtech/Materials/Default/Chemtech_ChemtechDecal";
    // 1. 根据材质名查找 texturePath
    let binhash = LeagueLoader::compute_binhash(&submesh.material_name.text);
    if is_debug {
        // println!("binhash: {:x}", binhash);
    }
    for entry in &map_materials.entries {
        if entry.path.hash == binhash {
            // ... 你原来的材质查找逻辑 ...
            // 假设你找到了 texture_path: String
            if is_debug {
                // println!("found binhash: {:#?}", entry);
            }
            if let Some(texture_path) = find_texture_path_in_material_entry(entry) {
                if is_debug {
                    // println!("texture_path: {}", texture_path);
                }
                match league_loader.get_image_by_texture_path(&texture_path) {
                    Ok(image) => return image,
                    Err(e) => warn!("Failed to load texture {}: {}", texture_path, e),
                }
            }
        }
    }

    println!("没找到{}，加载备用贴图", submesh.material_name.text);
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
        if !(texture_name == "DiffuseTexture" || texture_name == "Diffuse_Texture") {
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

            // 位置 (12 bytes)
            let x_pos = f32::from_le_bytes(v_chunk[offset..offset + 4].try_into().unwrap());
            let y_pos = f32::from_le_bytes(v_chunk[offset + 4..offset + 8].try_into().unwrap());
            let z_pos = f32::from_le_bytes(v_chunk[offset + 8..offset + 12].try_into().unwrap());
            positions.push([x_pos, y_pos, -z_pos]); // 翻转Z轴以适应Bevy的右手坐标系
            offset += 12;

            // 法线 (12 bytes)
            let x_norm = f32::from_le_bytes(v_chunk[offset..offset + 4].try_into().unwrap());
            let y_norm = f32::from_le_bytes(v_chunk[offset + 4..offset + 8].try_into().unwrap());
            let z_norm = f32::from_le_bytes(v_chunk[offset + 8..offset + 12].try_into().unwrap());
            normals.push([x_norm, y_norm, -z_norm]); // 同样翻转Z轴
            offset += 12;

            // UV坐标 (8 bytes)
            let u = f32::from_le_bytes(v_chunk[offset..offset + 4].try_into().unwrap());
            let v = f32::from_le_bytes(v_chunk[offset + 4..offset + 8].try_into().unwrap());
            uvs.push([u, v]);
            offset += 8;

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

            // 解析可选属性
            if let Some(colors_vec) = colors.as_mut() {
                // 顶点颜色 (4 bytes, u8 -> f32 normalized)
                let color_u8: [u8; 4] = v_chunk[offset..offset + 4].try_into().unwrap();
                colors_vec.push([
                    color_u8[0] as f32 / 255.0,
                    color_u8[1] as f32 / 255.0,
                    color_u8[2] as f32 / 255.0,
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
                tangents_vec.push([tan_x, tan_y, -tan_z, tan_w]);
                // offset += 16; // 已是最后一个元素，无需增加offset
            }
        }

        // 6. 获取、解析并调整索引
        let index_start_byte = range.start_index as usize * 2; // u16 = 2 bytes
        let index_end_byte = index_start_byte + (range.index_count as usize * 2);
        let index_data_slice = self.index_buffer.get(index_start_byte..index_end_byte)?;

        let mut local_indices: Vec<u16> = index_data_slice
            .chunks_exact(2)
            .map(|bytes| u16::from_le_bytes(bytes.try_into().unwrap()))
            .map(|global_index| global_index - range.start_vertex as u16) // 将全局索引转换为局部索引
            .collect();

        // 7. 修正因Z轴翻转导致的三角形环绕顺序问题
        for tri_indices in local_indices.chunks_exact_mut(3) {
            tri_indices.swap(1, 2); // 从 [0, 1, 2] -> [0, 2, 1]
        }

        // 8. 创建 Bevy Mesh 并插入所有顶点属性
        let mut bevy_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        // bevy_mesh.insert_attribute(
        //     Mesh::ATTRIBUTE_JOINT_INDEX,
        //     VertexAttributeValues::Uint16x4(joint_indices),
        // );
        // bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, joint_weights);

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
