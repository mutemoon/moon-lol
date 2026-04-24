use std::borrow::Cow;
use std::collections::BTreeMap;

use gltf::accessor::DataType;
use gltf::json::accessor::{Accessor, GenericComponentType, Type};
use gltf::json::buffer::{Buffer, Target, View};
use gltf::json::image::Image;
use gltf::json::material::{Material, PbrBaseColorFactor, PbrMetallicRoughness, StrengthFactor};
use gltf::json::mesh::{Mesh, Mode, Primitive, Semantic};
use gltf::json::scene::Scene;
use gltf::json::texture::{Info, Texture};
use gltf::json::validation::{Checked, USize64};
use gltf::json::{Index, Node, Root};
use image::codecs::png::{CompressionType, FilterType};
use image::{ExtendedColorType, ImageEncoder};
use league_file::mesh_skinned::LeagueSkinnedMesh;
use league_file::texture::{LeagueTexture, LeagueTextureFormat};
use texpresso::Format;

use crate::utils::Error;

/// 将 LeagueTexture 解码为 PNG 格式
pub fn decode_texture_to_png(texture: &LeagueTexture) -> Option<Vec<u8>> {
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

    let mut png_data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        &mut png_data,
        CompressionType::Fast,
        FilterType::NoFilter,
    );
    encoder
        .write_image(
            &rgba_data,
            texture.width as u32,
            texture.height as u32,
            ExtendedColorType::Rgba8,
        )
        .ok()?;
    Some(png_data)
}

/// 将角色皮肤（网格 + 材质 + 贴图）导出为 GLB 文件
pub fn export_skin_to_glb(
    skinned_mesh: &LeagueSkinnedMesh,
    texture_png: Option<Vec<u8>>,
    output_path: &str,
) -> Result<(), Error> {
    let mut builder = SkinGltfBuilder::new();

    // 添加贴图和材质
    let material_index = builder.add_material(texture_png);

    // 为每个 submesh 创建一个 primitive
    let mut primitives = Vec::new();
    for (i, _range) in skinned_mesh.ranges.iter().enumerate() {
        let primitive = builder.create_primitive(skinned_mesh, i, material_index)?;
        primitives.push(primitive);
    }

    // 如果没有 ranges（version 0），整个 mesh 作为一个 primitive
    if skinned_mesh.ranges.is_empty() {
        let primitive = builder.create_full_mesh_primitive(skinned_mesh, material_index)?;
        primitives.push(primitive);
    }

    if primitives.is_empty() {
        return Err(Error::Parse("没有可导出的网格数据".to_string()));
    }

    // 创建 mesh 和 node
    let mesh = Mesh {
        name: Some("skin_mesh".to_string()),
        primitives,
        extensions: None,
        extras: Default::default(),
        weights: None,
    };
    builder.meshes.push(mesh);

    let node = Node {
        name: Some("skin_node".to_string()),
        camera: None,
        children: None,
        extensions: None,
        extras: Default::default(),
        matrix: None,
        mesh: Some(Index::new(0)),
        rotation: None,
        scale: None,
        translation: None,
        weights: None,
        skin: None,
    };
    builder.nodes.push(node);

    builder.write_to_glb(output_path)
}

struct SkinGltfBuilder {
    buffer_data: Vec<u8>,
    accessors: Vec<Accessor>,
    buffer_views: Vec<View>,
    meshes: Vec<Mesh>,
    nodes: Vec<Node>,
    images: Vec<Image>,
    textures: Vec<Texture>,
    materials: Vec<Material>,
}

impl SkinGltfBuilder {
    fn new() -> Self {
        Self {
            buffer_data: Vec::new(),
            accessors: Vec::new(),
            buffer_views: Vec::new(),
            meshes: Vec::new(),
            nodes: Vec::new(),
            images: Vec::new(),
            textures: Vec::new(),
            materials: Vec::new(),
        }
    }

    fn align_to_4(&mut self) {
        let padding = (4 - (self.buffer_data.len() % 4)) % 4;
        if padding > 0 {
            self.buffer_data.resize(self.buffer_data.len() + padding, 0);
        }
    }

    /// 添加材质（可选贴图），返回材质索引
    fn add_material(&mut self, texture_png: Option<Vec<u8>>) -> u32 {
        let mut pbr = PbrMetallicRoughness {
            base_color_factor: PbrBaseColorFactor([1.0, 1.0, 1.0, 1.0]),
            base_color_texture: None,
            metallic_factor: StrengthFactor(0.0),
            roughness_factor: StrengthFactor(1.0),
            metallic_roughness_texture: None,
            extensions: None,
            extras: Default::default(),
        };

        if let Some(png_data) = texture_png {
            if !png_data.is_empty() {
                self.align_to_4();
                let offset = self.buffer_data.len();
                self.buffer_data.extend_from_slice(&png_data);

                let view_idx = self.buffer_views.len() as u32;
                self.buffer_views.push(View {
                    name: Some("texture_view".to_string()),
                    buffer: Index::new(0),
                    byte_offset: Some(USize64(offset as u64)),
                    byte_length: USize64(png_data.len() as u64),
                    byte_stride: None,
                    target: None,
                    extensions: None,
                    extras: Default::default(),
                });

                let image_idx = self.images.len() as u32;
                self.images.push(Image {
                    name: Some("skin_texture".to_string()),
                    uri: None,
                    buffer_view: Some(Index::new(view_idx)),
                    mime_type: Some(gltf::json::image::MimeType("image/png".to_string())),
                    extensions: None,
                    extras: Default::default(),
                });

                let tex_idx = self.textures.len() as u32;
                self.textures.push(Texture {
                    name: Some("skin_tex".to_string()),
                    sampler: None,
                    source: Index::new(image_idx),
                    extensions: None,
                    extras: Default::default(),
                });

                pbr.base_color_texture = Some(Info {
                    index: Index::new(tex_idx),
                    tex_coord: 0,
                    extensions: None,
                    extras: Default::default(),
                });
            }
        }

        let material_idx = self.materials.len() as u32;
        self.materials.push(Material {
            name: Some("skin_material".to_string()),
            pbr_metallic_roughness: pbr,
            normal_texture: None,
            occlusion_texture: None,
            emissive_texture: None,
            emissive_factor: Default::default(),
            alpha_mode: Checked::Valid(gltf::json::material::AlphaMode::Mask),
            alpha_cutoff: Some(gltf::json::material::AlphaCutoff(0.3)),
            double_sided: false,
            extensions: None,
            extras: Default::default(),
        });

        material_idx
    }

    /// 为指定 submesh 创建 primitive
    fn create_primitive(
        &mut self,
        skinned_mesh: &LeagueSkinnedMesh,
        submesh_index: usize,
        material_index: u32,
    ) -> Result<Primitive, Error> {
        let range = &skinned_mesh.ranges[submesh_index];
        let vertex_size = skinned_mesh.vertex_declaration.get_vertex_size() as usize;

        // 解析顶点数据
        let vertex_start = range.start_vertex as usize * vertex_size;
        let vertex_end = vertex_start + (range.vertex_count as usize * vertex_size);
        let vertex_slice = &skinned_mesh.vertex_buffer[vertex_start..vertex_end];

        let (positions, normals, uvs) = Self::parse_vertices(vertex_slice, vertex_size);

        // 解析索引数据
        let index_start = range.start_index as usize * 2;
        let index_end = index_start + (range.index_count as usize * 2);
        let index_slice = &skinned_mesh.index_buffer[index_start..index_end];

        let indices: Vec<u16> = index_slice
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes(b.try_into().unwrap()))
            .map(|idx| idx - range.start_vertex as u16)
            .collect();

        // 反转三角形面序
        let indices = Self::reverse_winding(&indices);

        self.build_primitive(&positions, &normals, &uvs, &indices, material_index)
    }

    /// 为整个 mesh 创建 primitive（version 0 没有 ranges）
    fn create_full_mesh_primitive(
        &mut self,
        skinned_mesh: &LeagueSkinnedMesh,
        material_index: u32,
    ) -> Result<Primitive, Error> {
        let vertex_size = skinned_mesh.vertex_declaration.get_vertex_size() as usize;
        let (positions, normals, uvs) =
            Self::parse_vertices(&skinned_mesh.vertex_buffer, vertex_size);

        let indices: Vec<u16> = skinned_mesh
            .index_buffer
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes(b.try_into().unwrap()))
            .collect();

        let indices = Self::reverse_winding(&indices);

        self.build_primitive(&positions, &normals, &uvs, &indices, material_index)
    }

    /// 从顶点 buffer 解析 position/normal/uv
    fn parse_vertices(
        vertex_data: &[u8],
        vertex_size: usize,
    ) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>) {
        let vertex_count = vertex_data.len() / vertex_size;
        let mut positions = Vec::with_capacity(vertex_count);
        let mut normals = Vec::with_capacity(vertex_count);
        let mut uvs = Vec::with_capacity(vertex_count);

        for chunk in vertex_data.chunks_exact(vertex_size) {
            // Position: offset 0, 12 bytes
            let px = f32::from_le_bytes(chunk[0..4].try_into().unwrap());
            let py = f32::from_le_bytes(chunk[4..8].try_into().unwrap());
            let pz = f32::from_le_bytes(chunk[8..12].try_into().unwrap());
            positions.push([px, py, pz]);

            // 跳过 bone_indices(4) + bone_weights(16) = 20 bytes
            let normal_offset = 32;
            let nx =
                f32::from_le_bytes(chunk[normal_offset..normal_offset + 4].try_into().unwrap());
            let ny = f32::from_le_bytes(
                chunk[normal_offset + 4..normal_offset + 8]
                    .try_into()
                    .unwrap(),
            );
            let nz = f32::from_le_bytes(
                chunk[normal_offset + 8..normal_offset + 12]
                    .try_into()
                    .unwrap(),
            );
            normals.push([nx, ny, nz]);

            // UV: offset 44, 8 bytes
            let uv_offset = 44;
            let u = f32::from_le_bytes(chunk[uv_offset..uv_offset + 4].try_into().unwrap());
            let v = f32::from_le_bytes(chunk[uv_offset + 4..uv_offset + 8].try_into().unwrap());
            uvs.push([u, v]);
        }

        (positions, normals, uvs)
    }

    /// 反转三角形面序（顺时针 → 逆时针）
    fn reverse_winding(indices: &[u16]) -> Vec<u16> {
        let mut result = Vec::with_capacity(indices.len());
        for tri in indices.chunks_exact(3) {
            result.push(tri[0]);
            result.push(tri[2]);
            result.push(tri[1]);
        }
        result
    }

    /// 构建 GLTF Primitive
    fn build_primitive(
        &mut self,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        uvs: &[[f32; 2]],
        indices: &[u16],
        material_index: u32,
    ) -> Result<Primitive, Error> {
        let mut attributes = BTreeMap::new();

        // Position accessor
        let pos_accessor_idx = self.add_vec3_accessor(positions, true);
        attributes.insert(
            Checked::Valid(Semantic::Positions),
            Index::new(pos_accessor_idx),
        );

        // Normal accessor
        let norm_accessor_idx = self.add_vec3_accessor(normals, false);
        attributes.insert(
            Checked::Valid(Semantic::Normals),
            Index::new(norm_accessor_idx),
        );

        // UV accessor
        let uv_accessor_idx = self.add_vec2_accessor(uvs);
        attributes.insert(
            Checked::Valid(Semantic::TexCoords(0)),
            Index::new(uv_accessor_idx),
        );

        // Index accessor
        let idx_accessor_idx = self.add_index_accessor(indices);

        Ok(Primitive {
            attributes,
            indices: Some(Index::new(idx_accessor_idx)),
            material: Some(Index::new(material_index)),
            mode: Checked::Valid(Mode::Triangles),
            targets: None,
            extensions: None,
            extras: Default::default(),
        })
    }

    fn add_vec3_accessor(&mut self, data: &[[f32; 3]], compute_bounds: bool) -> u32 {
        self.align_to_4();
        let offset = self.buffer_data.len();

        for v in data {
            self.buffer_data.extend_from_slice(&v[0].to_le_bytes());
            self.buffer_data.extend_from_slice(&v[1].to_le_bytes());
            self.buffer_data.extend_from_slice(&v[2].to_le_bytes());
        }

        let byte_length = data.len() * 12;
        let view_idx = self.buffer_views.len() as u32;
        self.buffer_views.push(View {
            name: None,
            buffer: Index::new(0),
            byte_offset: Some(USize64(offset as u64)),
            byte_length: USize64(byte_length as u64),
            byte_stride: None,
            target: Some(Checked::Valid(Target::ArrayBuffer)),
            extensions: None,
            extras: Default::default(),
        });

        let (min, max) = if compute_bounds {
            let mut min = [f32::MAX; 3];
            let mut max = [f32::MIN; 3];
            for v in data {
                for i in 0..3 {
                    min[i] = min[i].min(v[i]);
                    max[i] = max[i].max(v[i]);
                }
            }
            (
                Some(serde_json::to_value(min.to_vec()).unwrap()),
                Some(serde_json::to_value(max.to_vec()).unwrap()),
            )
        } else {
            (None, None)
        };

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(Accessor {
            buffer_view: Some(Index::new(view_idx)),
            byte_offset: Some(USize64(0)),
            component_type: Checked::Valid(GenericComponentType(DataType::F32)),
            count: USize64(data.len() as u64),
            type_: Checked::Valid(Type::Vec3),
            extensions: None,
            extras: Default::default(),
            min,
            max,
            name: None,
            normalized: false,
            sparse: None,
        });

        accessor_idx
    }

    fn add_vec2_accessor(&mut self, data: &[[f32; 2]]) -> u32 {
        self.align_to_4();
        let offset = self.buffer_data.len();

        for v in data {
            self.buffer_data.extend_from_slice(&v[0].to_le_bytes());
            self.buffer_data.extend_from_slice(&v[1].to_le_bytes());
        }

        let byte_length = data.len() * 8;
        let view_idx = self.buffer_views.len() as u32;
        self.buffer_views.push(View {
            name: None,
            buffer: Index::new(0),
            byte_offset: Some(USize64(offset as u64)),
            byte_length: USize64(byte_length as u64),
            byte_stride: None,
            target: Some(Checked::Valid(Target::ArrayBuffer)),
            extensions: None,
            extras: Default::default(),
        });

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(Accessor {
            buffer_view: Some(Index::new(view_idx)),
            byte_offset: Some(USize64(0)),
            component_type: Checked::Valid(GenericComponentType(DataType::F32)),
            count: USize64(data.len() as u64),
            type_: Checked::Valid(Type::Vec2),
            extensions: None,
            extras: Default::default(),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        accessor_idx
    }

    fn add_index_accessor(&mut self, data: &[u16]) -> u32 {
        self.align_to_4();
        let offset = self.buffer_data.len();

        for &idx in data {
            self.buffer_data.extend_from_slice(&idx.to_le_bytes());
        }

        let byte_length = data.len() * 2;
        let view_idx = self.buffer_views.len() as u32;
        self.buffer_views.push(View {
            name: None,
            buffer: Index::new(0),
            byte_offset: Some(USize64(offset as u64)),
            byte_length: USize64(byte_length as u64),
            byte_stride: None,
            target: Some(Checked::Valid(Target::ElementArrayBuffer)),
            extensions: None,
            extras: Default::default(),
        });

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(Accessor {
            buffer_view: Some(Index::new(view_idx)),
            byte_offset: Some(USize64(0)),
            component_type: Checked::Valid(GenericComponentType(DataType::U16)),
            count: USize64(data.len() as u64),
            type_: Checked::Valid(Type::Scalar),
            extensions: None,
            extras: Default::default(),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        accessor_idx
    }

    fn write_to_glb(self, output_path: &str) -> Result<(), Error> {
        let buffer = Buffer {
            name: Some("skin_buffer".to_string()),
            byte_length: USize64(self.buffer_data.len() as u64),
            uri: None,
            extensions: None,
            extras: Default::default(),
        };

        let node_indices: Vec<Index<Node>> = (0..self.nodes.len())
            .map(|i| Index::new(i as u32))
            .collect();

        let scene = Scene {
            name: Some("skin_scene".to_string()),
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
            materials: self.materials,
            textures: self.textures,
            images: self.images,
            scene: Some(Index::new(0)),
            scenes: vec![scene],
            ..Default::default()
        };

        let json_string = serde_json::to_string(&gltf_json)
            .map_err(|e| Error::Parse(format!("序列化 GLTF JSON 失败: {}", e)))?;
        let json_bytes = json_string.as_bytes();

        let mut glb_buffer = self.buffer_data;
        let padding = (4 - (glb_buffer.len() % 4)) % 4;
        for _ in 0..padding {
            glb_buffer.push(0);
        }

        let glb = gltf::binary::Glb {
            header: gltf::binary::Header {
                magic: *b"glTF",
                version: 2,
                length: 0,
            },
            json: Cow::Borrowed(json_bytes),
            bin: Some(Cow::Borrowed(&glb_buffer)),
        };

        let glb_data = glb
            .to_vec()
            .map_err(|e| Error::Parse(format!("生成 GLB 失败: {}", e)))?;

        let path = std::path::Path::new(output_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Parse(format!("创建目录失败: {}", e)))?;
        }

        std::fs::write(output_path, glb_data)
            .map_err(|e| Error::Parse(format!("写入 GLB 失败: {}", e)))?;

        Ok(())
    }
}
