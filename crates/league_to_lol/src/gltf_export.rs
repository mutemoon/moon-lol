use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

use gltf::accessor::DataType;
use gltf::json::accessor::{Accessor, GenericComponentType, Type};
use gltf::json::buffer::{Buffer, Stride, Target, View};
use gltf::json::image::Image;
use gltf::json::material::{Material, PbrBaseColorFactor, PbrMetallicRoughness, StrengthFactor};
use gltf::json::mesh::{Mesh, Mode, Primitive, Semantic};
use gltf::json::scene::Scene;
use gltf::json::texture::{Info, Texture};
use gltf::json::validation::{Checked, USize64};
use gltf::json::{Index, Node, Root};
use league_core::extract::StaticMaterialDef;
use league_core::mapgeo::EnvironmentVisibility;
use league_file::mapgeo::{
    ElementName, LeagueMapGeo, LeagueMapGeoMesh, Submesh, TextureOverride, VertexDeclaration,
};
use league_file::texture::{LeagueTexture, LeagueTextureFormat};
use league_loader::game::LeagueLoader;
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_utils::hash_bin;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;
use texpresso::Format;

use crate::utils::Error;

/// 将 LeagueMapGeo 导出为 gltf 文件
pub fn export_mapgeo_to_gltf(
    league_mapgeo: &LeagueMapGeo,
    output_path: &str,
    material_defs: &HashMap<u32, StaticMaterialDef>,
    loader: &LeagueLoader,
) -> Result<(), Error> {
    let start_total = Instant::now();

    // 1. 确定要导出的 Meshes
    // 这里可以根据需要调整：.take(1) 只导出第一个，或者删除 .take(1) 导出全部
    let target_meshes: Vec<_> = league_mapgeo
        .meshes
        .iter()
        .filter(|v| {
            v.environment_visibility
                .contains(EnvironmentVisibility::Layer1)
        })
        .collect();

    // 2. 从选定的 Mesh 中收集贴图路径，并过滤掉 Diffuse 贴图缺失的模型
    let start_collect = Instant::now();
    let mut texture_paths = std::collections::HashSet::new();
    let mut existence_cache = HashMap::new();
    let target_meshes: Vec<_> = target_meshes
        .into_iter()
        .filter(|map_mesh| {
            // 检查每个子网格实际使用的 Diffuse 贴图是否存在
            for submesh in &map_mesh.submeshes {
                let material_hash = hash_bin(&submesh.material_name.text);

                // 1. 获取材质定义中的 Diffuse 信息 (sampler_idx, def_path)
                let diffuse_info = material_defs.get(&material_hash).and_then(|def| {
                    def.sampler_values
                        .as_ref()?
                        .iter()
                        .enumerate()
                        .find_map(|(idx, s)| {
                            if (s.texture_name == "DiffuseTexture"
                                || s.texture_name == "Diffuse_Texture")
                                && s.texture_path.is_some()
                            {
                                Some((idx as u32, s.texture_path.as_ref().unwrap().clone()))
                            } else {
                                None
                            }
                        })
                });

                // 2. 结合覆盖逻辑确定最终路径
                let resolved_path = match diffuse_info {
                    Some((sampler_idx, def_path)) => map_mesh
                        .texture_overrides
                        .iter()
                        .find(|o| o.sampler_id == sampler_idx)
                        .map(|o| o.texture_path.text.clone())
                        .or(Some(def_path)),
                    None => map_mesh
                        .texture_overrides
                        .iter()
                        .find(|o| o.sampler_id == 0)
                        .map(|o| o.texture_path.text.clone()),
                };

                // 3. 检查解析出的 Diffuse 贴图是否存在
                match resolved_path {
                    Some(path) => {
                        let exists = *existence_cache.entry(path.clone()).or_insert_with(|| {
                            let hash = league_utils::hash_wad(&path.to_lowercase());
                            loader
                                .wads
                                .iter()
                                .any(|wad| wad.wad.get_entry(hash).is_ok())
                        });

                        if !exists {
                            println!(
                                "⚠️ 警告: 材质 {} 缺失 Diffuse 贴图 {}, 跳过导出该模型",
                                submesh.material_name.text, path
                            );
                            return false;
                        }
                    }
                    None => {
                        println!(
                            "⚠️ 警告: 材质 {} 未定义 Diffuse 贴图, 跳过导出该模型",
                            submesh.material_name.text
                        );
                        return false;
                    }
                }
            }

            // 如果 Diffuse 检查通过，收集该模型关联的所有贴图路径（用于后续 Step 3 处理）
            for submesh in &map_mesh.submeshes {
                let material_hash = hash_bin(&submesh.material_name.text);
                if let Some(def) = material_defs.get(&material_hash) {
                    if let Some(samplers) = &def.sampler_values {
                        for sampler in samplers {
                            if (sampler.texture_name == "DiffuseTexture"
                                || sampler.texture_name == "Diffuse_Texture")
                                && sampler.texture_path.is_some()
                            {
                                texture_paths
                                    .insert(sampler.texture_path.as_ref().unwrap().clone());
                            }
                        }
                    }
                }
            }
            for texture_override in &map_mesh.texture_overrides {
                texture_paths.insert(texture_override.texture_path.text.clone());
            }
            true
        })
        .collect();
    let duration_collect = start_collect.elapsed();

    let texture_paths: Vec<_> = texture_paths.into_iter().collect();
    if !texture_paths.is_empty() {
        println!("📦 正在并行处理 {} 个贴图...", texture_paths.len());
    }

    // 3. 并行处理纹理
    let start_textures = Instant::now();
    #[cfg(not(target_arch = "wasm32"))]
    let texture_iter = texture_paths.par_iter();
    #[cfg(target_arch = "wasm32")]
    let texture_iter = texture_paths.iter();

    let processed_textures: HashMap<String, Vec<u8>> = texture_iter
        .filter_map(|path| {
            let buf = loader.get_wad_entry_buffer_by_path(path).ok()?;
            let (_, texture) = LeagueTexture::parse(&buf).ok()?;

            let format = match texture.format {
                LeagueTextureFormat::Bc1 => Some(Format::Bc1),
                LeagueTextureFormat::Bc3 => Some(Format::Bc3),
                LeagueTextureFormat::Bgra8 => None,
                _ => return None,
            };

            let rgba_data = if let Some(f) = format {
                let mut rgba = vec![0u8; texture.width as usize * texture.height as usize * 4];
                f.decompress(
                    &texture.mipmaps[0],
                    texture.width as usize,
                    texture.height as usize,
                    &mut rgba,
                );
                rgba
            } else if texture.format == LeagueTextureFormat::Bgra8 {
                let mut data = texture.mipmaps[0].clone();
                for chunk in data.chunks_exact_mut(4) {
                    chunk.swap(0, 2);
                }
                data
            } else {
                return None;
            };

            let png_data = {
                let mut png_data = Vec::new();
                let encoder = image::codecs::png::PngEncoder::new_with_quality(
                    &mut png_data,
                    image::codecs::png::CompressionType::Fast,
                    image::codecs::png::FilterType::NoFilter,
                );
                use image::ImageEncoder;
                encoder
                    .write_image(
                        &rgba_data,
                        texture.width as u32,
                        texture.height as u32,
                        image::ExtendedColorType::Rgba8,
                    )
                    .ok()?;
                png_data
            };

            Some((path.clone(), png_data))
        })
        .collect();
    let duration_textures = start_textures.elapsed();

    // 4. 构建 GLTF (仅添加选定的 Mesh)
    let start_build = Instant::now();
    let mut gltf_data = GltfBuilder::new(material_defs, processed_textures);

    for map_mesh in target_meshes {
        gltf_data.add_mesh(league_mapgeo, map_mesh)?;
    }

    let duration_build = start_build.elapsed();

    // 4. 写入文件
    let start_write = Instant::now();
    gltf_data.write_to_glb(output_path)?;
    let duration_write = start_write.elapsed();

    let total_duration = start_total.elapsed();

    println!("----------------------------------------");
    println!("⏱️  导出耗时统计:");
    println!("  - 路径收集:   {:?}", duration_collect);
    println!("  - 纹理并行处理: {:?}", duration_textures);
    println!("  - GLTF 数据构建: {:?}", duration_build);
    println!("  - 文件 I/O 写入: {:?}", duration_write);
    println!("  - 总计耗时:     {:?}", total_duration);
    println!("----------------------------------------");

    Ok(())
}

struct GltfBuilder<'a> {
    buffer_data: Vec<u8>,
    accessors: Vec<Accessor>,
    buffer_views: Vec<View>,
    meshes: Vec<Mesh>,
    nodes: Vec<Node>,
    images: Vec<Image>,
    textures: Vec<Texture>,
    materials: Vec<Material>,
    material_cache: HashMap<String, u32>,
    image_cache: HashMap<String, u32>,
    material_defs: &'a HashMap<u32, StaticMaterialDef>,
    processed_textures: HashMap<String, Vec<u8>>,
    vbuf_views: HashMap<u32, u32>,            // vbuf_idx -> view_idx
    ibuf_views: HashMap<u32, u32>,            // ibuf_idx -> view_idx
    accessor_cache: HashMap<(u32, u32), u32>, // (vbuf_idx, element_offset_in_decl) -> accessor_idx
}

impl<'a> GltfBuilder<'a> {
    fn new(
        material_defs: &'a HashMap<u32, StaticMaterialDef>,
        processed_textures: HashMap<String, Vec<u8>>,
    ) -> Self {
        Self {
            buffer_data: Vec::new(),
            accessors: Vec::new(),
            buffer_views: Vec::new(),
            meshes: Vec::new(),
            nodes: Vec::new(),
            images: Vec::new(),
            textures: Vec::new(),
            materials: Vec::new(),
            material_cache: HashMap::new(),
            image_cache: HashMap::new(),
            material_defs,
            processed_textures,
            vbuf_views: HashMap::new(),
            ibuf_views: HashMap::new(),
            accessor_cache: HashMap::new(),
        }
    }

    fn get_or_import_vbuf(
        &mut self,
        league_mapgeo: &LeagueMapGeo,
        vbuf_idx: u32,
        _declaration: &VertexDeclaration,
    ) -> u32 {
        if let Some(&view_idx) = self.vbuf_views.get(&vbuf_idx) {
            return view_idx;
        }

        self.align_to_4();
        let offset = self.buffer_data.len();
        let vbuf = &league_mapgeo.vertex_buffers[vbuf_idx as usize];

        self.buffer_data.extend_from_slice(&vbuf.buffer);

        let view_idx = self.buffer_views.len() as u32;
        self.buffer_views.push(View {
            name: Some(format!("vbuf_{}", vbuf_idx)),
            buffer: Index::new(0),
            byte_offset: Some(USize64(offset as u64)),
            byte_length: USize64(vbuf.buffer.len() as u64),
            byte_stride: None, // 稍后在 Accessor 层面由声明决定
            target: Some(Checked::Valid(Target::ArrayBuffer)),
            extensions: None,
            extras: Default::default(),
        });
        self.vbuf_views.insert(vbuf_idx, view_idx);
        view_idx
    }

    fn get_or_import_ibuf(&mut self, league_mapgeo: &LeagueMapGeo, ibuf_idx: u32) -> u32 {
        if let Some(&view_idx) = self.ibuf_views.get(&ibuf_idx) {
            return view_idx;
        }

        self.align_to_4();
        let offset = self.buffer_data.len();
        let ibuf = &league_mapgeo.index_buffers[ibuf_idx as usize];
        for &idx in &ibuf.buffer {
            self.buffer_data.extend_from_slice(&idx.to_le_bytes());
        }

        let view_idx = self.buffer_views.len() as u32;
        self.buffer_views.push(View {
            name: Some(format!("ibuf_{}", ibuf_idx)),
            buffer: Index::new(0),
            byte_offset: Some(USize64(offset as u64)),
            byte_length: USize64((ibuf.buffer.len() * 2) as u64),
            byte_stride: None,
            target: Some(Checked::Valid(Target::ElementArrayBuffer)),
            extensions: None,
            extras: Default::default(),
        });
        self.ibuf_views.insert(ibuf_idx, view_idx);
        view_idx
    }

    fn align_to_4(&mut self) {
        let padding = (4 - (self.buffer_data.len() % 4)) % 4;
        if padding > 0 {
            self.buffer_data.resize(self.buffer_data.len() + padding, 0);
        }
    }

    fn add_mesh(
        &mut self,
        league_mapgeo: &LeagueMapGeo,
        map_mesh: &LeagueMapGeoMesh,
    ) -> Result<(), Error> {
        let mut primitives = Vec::new();

        for submesh in &map_mesh.submeshes {
            let decl_idx = map_mesh.vertex_declaration_index_base as usize;
            let declaration = &league_mapgeo.vertex_declarations[decl_idx];
            let has_uv = declaration
                .elements
                .iter()
                .any(|e| e.name == ElementName::Texcoord0);

            let material_index = self.get_or_create_material(
                &submesh.material_name.text,
                &map_mesh.texture_overrides,
                has_uv,
            );
            let primitive =
                self.create_primitive(league_mapgeo, map_mesh, submesh, material_index)?;
            primitives.push(primitive);
        }

        if !primitives.is_empty() {
            let mesh_index = self.meshes.len();
            let mesh = Mesh {
                name: Some(format!("mesh_{}", mesh_index)),
                primitives,
                extensions: None,
                extras: Default::default(),
                weights: None,
            };
            self.meshes.push(mesh);

            let transform_matrix: [f32; 16] = map_mesh.transform.clone().try_into().unwrap_or([
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ]);

            let node = Node {
                name: Some(format!("node_{}", mesh_index)),
                camera: None,
                children: None,
                extensions: None,
                extras: Default::default(),
                matrix: Some(transform_matrix),
                mesh: Some(Index::new(mesh_index as u32)),
                rotation: None,
                scale: None,
                translation: None,
                weights: None,
                skin: None,
            };
            self.nodes.push(node);
        }

        Ok(())
    }

    fn create_primitive(
        &mut self,
        map_file: &LeagueMapGeo,
        map_mesh: &LeagueMapGeoMesh,
        submesh: &Submesh,
        material_index: u32,
    ) -> Result<Primitive, Error> {
        let decl_idx = map_mesh.vertex_declaration_index_base as usize;
        let declaration = &map_file.vertex_declarations[decl_idx];

        let mut attributes = BTreeMap::new();
        let mut stride = 0;
        for element in &declaration.elements {
            stride += element.format.get_size();
        }

        let mut current_offset = 0;
        for element in &declaration.elements {
            let size = element.format.get_size();
            let semantic = match element.name {
                ElementName::Position => Some(Semantic::Positions),
                ElementName::Normal => Some(Semantic::Normals),
                ElementName::Texcoord0 => Some(Semantic::TexCoords(0)),
                _ => None,
            };

            if let Some(s) = semantic {
                let vbuf_idx = map_mesh.vertex_buffer_indexes[0];
                let view_idx = self.get_or_import_vbuf(map_file, vbuf_idx, declaration);

                let accessor_idx = if let Some(&idx) =
                    self.accessor_cache.get(&(vbuf_idx, current_offset as u32))
                {
                    idx
                } else {
                    let idx = self.accessors.len() as u32;
                    let vbuf = &map_file.vertex_buffers[vbuf_idx as usize];
                    let count = vbuf.buffer.len() / stride;

                    let mut accessor = Accessor {
                        buffer_view: Some(Index::new(view_idx)),
                        byte_offset: Some(USize64(current_offset as u64)),
                        component_type: Checked::Valid(GenericComponentType(DataType::F32)),
                        count: USize64(count as u64),
                        type_: match element.name {
                            ElementName::Position | ElementName::Normal => {
                                Checked::Valid(Type::Vec3)
                            }
                            ElementName::Texcoord0 => Checked::Valid(Type::Vec2),
                            _ => Checked::Valid(Type::Scalar),
                        },
                        extensions: None,
                        extras: Default::default(),
                        min: None,
                        max: None,
                        name: None,
                        normalized: false,
                        sparse: None,
                    };

                    if element.name == ElementName::Position {
                        let mut min = [f32::MAX; 3];
                        let mut max = [f32::MIN; 3];

                        for i in 0..count {
                            let offset = i * stride + current_offset;
                            let bytes = &vbuf.buffer[offset..offset + 12];
                            let x = f32::from_le_bytes(bytes[0..4].try_into().unwrap());
                            let y = f32::from_le_bytes(bytes[4..8].try_into().unwrap());
                            let z = f32::from_le_bytes(bytes[8..12].try_into().unwrap());

                            min[0] = min[0].min(x);
                            min[1] = min[1].min(y);
                            min[2] = min[2].min(z);

                            max[0] = max[0].max(x);
                            max[1] = max[1].max(y);
                            max[2] = max[2].max(z);
                        }

                        accessor.min = Some(serde_json::to_value(min.to_vec()).unwrap());
                        accessor.max = Some(serde_json::to_value(max.to_vec()).unwrap());
                    }

                    self.accessors.push(accessor);

                    if self.buffer_views[view_idx as usize].byte_stride.is_none() {
                        self.buffer_views[view_idx as usize].byte_stride = Some(Stride(stride));
                    }

                    self.accessor_cache
                        .insert((vbuf_idx, current_offset as u32), idx);
                    idx
                };

                attributes.insert(Checked::Valid(s), Index::new(accessor_idx));
            }
            current_offset += size;
        }

        let ibuf_idx = map_mesh.index_buffer_id;
        let view_idx = self.get_or_import_ibuf(map_file, ibuf_idx);

        let index_accessor_idx = self.accessors.len() as u32;
        self.accessors.push(Accessor {
            buffer_view: Some(Index::new(view_idx)),
            byte_offset: Some(USize64((submesh.start_index * 2) as u64)),
            component_type: Checked::Valid(GenericComponentType(DataType::U16)),
            count: USize64(submesh.submesh_index_count as u64),
            type_: Checked::Valid(Type::Scalar),
            extensions: None,
            extras: Default::default(),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        let primitive = Primitive {
            attributes,
            indices: Some(Index::new(index_accessor_idx)),
            material: Some(Index::new(material_index)),
            mode: Checked::Valid(Mode::Triangles),
            targets: None,
            extensions: None,
            extras: Default::default(),
        };

        Ok(primitive)
    }

    fn get_or_create_material(
        &mut self,
        material_name: &str,
        texture_overrides: &[TextureOverride],
        has_uv: bool,
    ) -> u32 {
        let material_hash = hash_bin(material_name);
        let material_def_texture_path = self.find_diffuse_path(material_hash);

        let base_color_texture_path = match material_def_texture_path {
            Some((sampler_idx, def_path)) => texture_overrides
                .iter()
                .find(|o| o.sampler_id == sampler_idx)
                .map(|o| o.texture_path.text.clone())
                .or(Some(def_path.clone())),
            None => texture_overrides
                .iter()
                .find(|o| o.sampler_id == 0)
                .map(|o| o.texture_path.text.clone()),
        };

        // 如果没有 UV，即便有贴图路径也强制不使用纹理，以避免 glTF 验证错误
        let cache_key = if has_uv {
            match &base_color_texture_path {
                Some(path) => format!("{}_{}", material_name, path),
                None => material_name.to_string(),
            }
        } else {
            format!("{}_NoUV", material_name)
        };

        if let Some(&index) = self.material_cache.get(&cache_key) {
            return index;
        }

        let texture_index = if has_uv {
            self.load_texture_index(base_color_texture_path.as_ref())
        } else {
            0
        };
        let material_index = self.materials.len() as u32;

        let mut pbr = PbrMetallicRoughness {
            base_color_factor: PbrBaseColorFactor([1.0, 1.0, 1.0, 1.0]),
            base_color_texture: None,
            metallic_factor: StrengthFactor(0.0),
            roughness_factor: StrengthFactor(1.0),
            metallic_roughness_texture: None,
            extensions: None,
            extras: Default::default(),
        };

        if texture_index > 0 && texture_index < (self.textures.len() as u32 + 1) {
            pbr.base_color_texture = Some(Info {
                index: Index::new(texture_index - 1),
                tex_coord: 0,
                extensions: None,
                extras: Default::default(),
            });
        }

        self.materials.push(Material {
            name: Some(material_name.to_string()),
            pbr_metallic_roughness: pbr,
            normal_texture: None,
            occlusion_texture: None,
            emissive_texture: None,
            emissive_factor: Default::default(),
            alpha_mode: Checked::Valid(gltf::json::material::AlphaMode::Mask),
            alpha_cutoff: Some(gltf::json::material::AlphaCutoff(0.5)),
            double_sided: false,
            extensions: None,
            extras: Default::default(),
        });

        self.material_cache.insert(cache_key, material_index);
        material_index
    }

    fn find_diffuse_path(&self, material_hash: u32) -> Option<(u32, &String)> {
        let def = self.material_defs.get(&material_hash)?;
        let samplers = def.sampler_values.as_ref()?;
        samplers.iter().enumerate().find_map(|(idx, s)| {
            let is_diffuse =
                s.texture_name == "DiffuseTexture" || s.texture_name == "Diffuse_Texture";
            let path = s.texture_path.as_ref()?;
            if is_diffuse {
                Some((idx as u32, path))
            } else {
                None
            }
        })
    }

    fn load_texture_index(&mut self, path: Option<&String>) -> u32 {
        let Some(path) = path else { return 0 };
        if let Some(&index) = self.image_cache.get(path) {
            return index + 1;
        }

        if !self.processed_textures.contains_key(path) {
            return 0;
        }

        self.align_to_4();

        let png_data = self.processed_textures.get(path).unwrap();
        if png_data.is_empty() {
            return 0;
        }

        let offset = self.buffer_data.len();
        self.buffer_data.extend_from_slice(png_data);

        let buffer_view_index = self.buffer_views.len() as u32;
        self.buffer_views.push(View {
            name: Some(format!("img_view_{}", buffer_view_index)),
            buffer: Index::new(0),
            byte_offset: Some(USize64(offset as u64)),
            byte_length: USize64(png_data.len() as u64),
            byte_stride: None,
            target: None,
            extensions: None,
            extras: Default::default(),
        });

        let image_index = self.images.len() as u32;
        self.images.push(Image {
            name: Some(format!("img_{}", image_index)),
            uri: None,
            buffer_view: Some(Index::new(buffer_view_index)),
            mime_type: Some(gltf::json::image::MimeType("image/png".to_string())),
            extensions: None,
            extras: Default::default(),
        });

        let tex_idx = self.textures.len() as u32;
        self.textures.push(Texture {
            name: Some(format!("tex_{}", tex_idx)),
            sampler: None,
            source: Index::new(image_index),
            extensions: None,
            extras: Default::default(),
        });
        self.image_cache.insert(path.clone(), tex_idx);
        tex_idx + 1
    }

    fn write_to_glb(self, output_path: &str) -> Result<(), Error> {
        let materials = self.materials;

        let buffer = Buffer {
            name: Some("geometry_buffer".to_string()),
            byte_length: USize64(self.buffer_data.len() as u64),
            uri: None, // GLB 内嵌 Buffer 不需要 URI
            extensions: None,
            extras: Default::default(),
        };

        // 创建场景
        let node_indices: Vec<Index<Node>> = (0..self.nodes.len())
            .map(|i| Index::new(i as u32))
            .collect();
        let scene = Scene {
            name: Some("default_scene".to_string()),
            nodes: node_indices,
            extensions: None,
            extras: Default::default(),
        };

        let gltf_json = Root {
            asset: gltf::json::asset::Asset {
                version: "2.0".into(),
                ..Default::default()
            },
            buffers: vec![buffer],
            buffer_views: self.buffer_views,
            accessors: self.accessors,
            meshes: self.meshes,
            nodes: self.nodes,
            materials,
            textures: self.textures,
            images: self.images,
            scene: Some(Index::new(0)),
            scenes: vec![scene],
            ..Default::default()
        };

        // 生成 GLB 内容
        let json_string = serde_json::to_string(&gltf_json)
            .map_err(|e| Error::Parse(format!("Failed to serialize gltf JSON: {}", e)))?;
        let json_bytes = json_string.as_bytes();

        let mut glb_buffer = self.buffer_data;
        // 确保二进制数据是 4 字节对齐的
        let padding = (4 - (glb_buffer.len() % 4)) % 4;
        for _ in 0..padding {
            glb_buffer.push(0);
        }

        let glb = gltf::binary::Glb {
            header: gltf::binary::Header {
                magic: *b"glTF",
                version: 2,
                length: 0, // 会在 to_vec 中自动计算
            },
            json: Cow::Borrowed(json_bytes),
            bin: Some(Cow::Borrowed(&glb_buffer)),
        };

        let glb_path = format!("{}.glb", output_path);
        let glb_data = glb
            .to_vec()
            .map_err(|e| Error::Parse(format!("Failed to generate GLB: {}", e)))?;

        std::fs::write(&glb_path, glb_data)
            .map_err(|e| Error::Parse(format!("Failed to write GLB file: {}", e)))?;

        println!("✅ GLB 导出成功: {}", glb_path);

        Ok(())
    }
}
