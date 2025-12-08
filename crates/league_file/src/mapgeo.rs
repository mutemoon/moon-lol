use bevy::prelude::*;
use binrw::binread;
use league_core::{EnvironmentVisibility, LayerTransitionBehavior};
use league_utils::{parse_vec3_array, BoundingBox};
use serde::{Deserialize, Serialize};

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
#[br(magic = b"OEGM")]
#[br(little)]
pub struct LeagueMapGeo {
    pub version: u32,

    pub sampler_count: u32,
    #[br(count = sampler_count)]
    pub samplers: Vec<ShaderTextureOverride>,

    pub vertex_declaration_count: u32,
    #[br(count = vertex_declaration_count)]
    pub vertex_declarations: Vec<VertexDeclaration>,

    pub vertex_buffer_count: u32,
    #[br(count = vertex_buffer_count)]
    pub vertex_buffers: Vec<VertexBuffer>,

    pub index_buffer_count: u32,
    #[br(count = index_buffer_count)]
    pub index_buffers: Vec<IndexBuffer>,

    pub mesh_count: u32,
    #[br(count = mesh_count)]
    pub meshes: Vec<LeagueMapGeoMesh>,

    pub scene_graph_count: u32,
    #[br(count = scene_graph_count)]
    pub scene_graphs: Vec<SceneGraph>,

    pub planar_reflector_count: u32,
    #[br(count = planar_reflector_count)]
    pub planar_reflectors: Vec<PlanarReflector>,
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct SceneGraph {
    pub visibility_controller_path_hash: u32,

    pub min_x: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_z: f32,
    pub max_stick_out_x: f32,
    pub max_stick_out_z: f32,
    pub bucket_size_x: f32,
    pub bucket_size_z: f32,

    pub buckets_per_side: u16,
    #[br(map = |v: u8| v != 0)]
    pub is_disabled: bool,
    #[br(map = EnvironmentVisibility::from_bits_truncate)]
    pub environment_visibility: EnvironmentVisibility,

    pub vertex_count: u32,
    pub index_count: u32,

    #[br(if(!is_disabled))]
    #[br(count = vertex_count)]
    #[br(map = parse_vec3_array)]
    pub vertices: Vec<Vec3>,

    #[br(if(!is_disabled))]
    #[br(count = index_count)]
    pub indices: Vec<u16>,

    #[br(if(!is_disabled))]
    #[br(count = buckets_per_side.pow(2))]
    pub buckets: Vec<GeometryBucket>,

    #[br(if(!is_disabled && environment_visibility != EnvironmentVisibility::NoLayer))]
    #[br(count = index_count / 3)]
    pub face_visibility_flags: Vec<u8>,
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct GeometryBucket {
    pub max_stick_out_x: f32,
    pub max_stick_out_z: f32,
    pub start_index: u32,
    pub base_vertex: u32,
    pub inside_face_count: u16,
    pub sticking_out_face_count: u16,
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct PlanarReflector {
    #[br(map = |v: [f32; 16]| Mat4::from_cols_array(&v))]
    pub transform: Mat4,
    pub bounds: BoundingBox,
    #[br(map = Vec3::from_array)]
    pub normal: Vec3,
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct ShaderTextureOverride {
    pub id: u32,
    pub path: SizedStringU32,
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct VertexDeclaration {
    pub usage: u32,

    pub element_count: u32,
    #[br(count = 15, map = |v: Vec<VertexElement>| (&v[..element_count as usize]).to_vec())]
    pub elements: Vec<VertexElement>,
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct VertexElement {
    pub name: ElementName,
    pub format: ElementFormat,
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct VertexBuffer {
    #[br(map = EnvironmentVisibility::from_bits_truncate)]
    pub environment_visibility: EnvironmentVisibility,

    pub buffer_count: u32,
    #[br(count = buffer_count)]
    pub buffer: Vec<u8>,
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct IndexBuffer {
    #[br(map = EnvironmentVisibility::from_bits_truncate)]
    pub environment_visibility: EnvironmentVisibility,

    pub buffer_count: u32,
    #[br(count = buffer_count / 2)]
    pub buffer: Vec<u16>,
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little, repr = u32)]
pub enum ElementFormat {
    Unknown = -1,
    XFloat32,
    XyFloat32,
    XyzFloat32,
    XyzwFloat32,
    BgraPacked8888,
    ZyxwPacked8888,
    RgbaPacked8888,
    XyPacked1616,
    XyzPacked161616,
    XyzwPacked16161616,
    XyPacked88,
    XyzPacked888,
    XyzwPacked8888,
}

impl ElementFormat {
    pub fn get_size(&self) -> usize {
        match self {
            ElementFormat::XFloat32 => 4,
            ElementFormat::XyFloat32 => 8,
            ElementFormat::XyzFloat32 => 12,
            ElementFormat::XyzwFloat32 => 16,
            ElementFormat::BgraPacked8888
            | ElementFormat::ZyxwPacked8888
            | ElementFormat::RgbaPacked8888 => 4,
            ElementFormat::XyPacked1616 => 4,
            ElementFormat::XyzPacked161616 => 8,
            ElementFormat::XyzwPacked16161616 => 8,
            ElementFormat::XyPacked88 => 2,
            ElementFormat::XyzPacked888 => 3,
            ElementFormat::XyzwPacked8888 => 4,
            ElementFormat::Unknown => 0,
        }
    }
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little, repr = u32)]
pub enum ElementName {
    Unknown = -1,
    Position,
    BlendWeight,
    Normal,
    FogCoordinate,
    PrimaryColor,
    SecondaryColor,
    BlendIndex,
    Texcoord0,
    Texcoord1,
    Texcoord2,
    Texcoord3,
    Texcoord4,
    Texcoord5,
    Texcoord6,
    Texcoord7,
    Tangent,
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct LeagueMapGeoMesh {
    pub vertex_count: u32,
    pub vertex_declaration_count: u32,
    pub vertex_declaration_index_base: u32,

    #[br(count = vertex_declaration_count)]
    pub vertex_buffer_indexes: Vec<u32>,

    pub index_count: u32,
    pub index_buffer_id: u32,

    #[br(map = EnvironmentVisibility::from_bits_truncate)]
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

    #[br(map = |v: u8| v.into())]
    pub quality_filter: QualityFilter,

    #[br(map = |v: u8| v.into())]
    pub layer_transition_behavior: LayerTransitionBehavior,

    pub render_flags: u16,

    pub baked_light: Channel,
    pub stationary_light: Channel,

    pub texture_override_count: u32,
    #[br(count = texture_override_count)]
    pub texture_overrides: Vec<TextureOverride>,

    #[br(map = Vec2::from_array)]
    pub baked_paint_scale: Vec2,
    #[br(map = Vec2::from_array)]
    pub baked_paint_bias: Vec2,
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct Submesh {
    pub hash: u32,

    pub material_name: SizedStringU32,

    pub start_index: u32,
    pub submesh_index_count: u32,
    pub min_vertex: u32,
    pub max_vertex: u32,
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct Channel {
    pub texture: SizedStringU32,
    #[br(map = Vec2::from_array)]
    pub uv_scale: Vec2,
    #[br(map = Vec2::from_array)]
    pub uv_offset: Vec2,
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct TextureOverride {
    pub sampler_id: u32,
    pub texture_path: SizedStringU32,
}

#[binread]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct SizedStringU32 {
    pub len: u32,
    #[br(count = len, try_map = |bytes: Vec<u8>| {
        // 找到第一个 null 字符，如果有的话就截断
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8(bytes[..end].to_vec())
    })]
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum QualityFilter {
    All,
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl From<u8> for QualityFilter {
    fn from(value: u8) -> Self {
        match value {
            1 => QualityFilter::VeryLow,
            2 => QualityFilter::Low,
            4 => QualityFilter::Medium,
            8 => QualityFilter::High,
            16 => QualityFilter::VeryHigh,
            _ => QualityFilter::All,
        }
    }
}
