use bevy::math::{Mat4, Vec2, Vec3};
use bevy::prelude::*;
use league_core::{EnvironmentVisibility, LayerTransitionBehavior};
use league_utils::BoundingBox;
use nom::bytes::complete::take;
use nom::multi::count;
use nom::number::complete::{le_f32, le_u16, le_u32, le_u8};
use nom::{IResult, Parser};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct LeagueMapGeo {
    pub version: u32,
    pub samplers: Vec<ShaderTextureOverride>,
    pub vertex_declarations: Vec<VertexDeclaration>,
    pub vertex_buffers: Vec<VertexBuffer>,
    pub index_buffers: Vec<IndexBuffer>,
    pub meshes: Vec<LeagueMapGeoMesh>,
    pub scene_graphs: Vec<SceneGraph>,
    pub planar_reflectors: Vec<PlanarReflector>,
}

impl LeagueMapGeo {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, _) = take(4usize)(input)?; // magic: OEGM
        let (i, version) = le_u32(i)?;

        let (i, sampler_count) = le_u32(i)?;
        let (i, samplers) = count(ShaderTextureOverride::parse, sampler_count as usize).parse(i)?;

        let (i, vertex_declaration_count) = le_u32(i)?;
        let (i, vertex_declarations) =
            count(VertexDeclaration::parse, vertex_declaration_count as usize).parse(i)?;

        let (i, vertex_buffer_count) = le_u32(i)?;
        let (i, vertex_buffers) =
            count(VertexBuffer::parse, vertex_buffer_count as usize).parse(i)?;

        let (i, index_buffer_count) = le_u32(i)?;
        let (i, index_buffers) = count(IndexBuffer::parse, index_buffer_count as usize).parse(i)?;

        let (i, mesh_count) = le_u32(i)?;
        let (i, meshes) = count(LeagueMapGeoMesh::parse, mesh_count as usize).parse(i)?;

        let (i, scene_graph_count) = le_u32(i)?;
        let (i, scene_graphs) = count(SceneGraph::parse, scene_graph_count as usize).parse(i)?;

        let (i, planar_reflector_count) = le_u32(i)?;
        let (i, planar_reflectors) =
            count(PlanarReflector::parse, planar_reflector_count as usize).parse(i)?;

        Ok((
            i,
            LeagueMapGeo {
                version,
                samplers,
                vertex_declarations,
                vertex_buffers,
                index_buffers,
                meshes,
                scene_graphs,
                planar_reflectors,
            },
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub is_disabled: bool,
    pub environment_visibility: EnvironmentVisibility,
    pub vertex_count: u32,
    pub index_count: u32,
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u16>,
    pub buckets: Vec<GeometryBucket>,
    pub face_visibility_flags: Vec<u8>,
}

impl SceneGraph {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, visibility_controller_path_hash) = le_u32(input)?;
        let (i, min_x) = le_f32(i)?;
        let (i, min_z) = le_f32(i)?;
        let (i, max_x) = le_f32(i)?;
        let (i, max_z) = le_f32(i)?;
        let (i, max_stick_out_x) = le_f32(i)?;
        let (i, max_stick_out_z) = le_f32(i)?;
        let (i, bucket_size_x) = le_f32(i)?;
        let (i, bucket_size_z) = le_f32(i)?;
        let (i, buckets_per_side) = le_u16(i)?;
        let (i, is_disabled_raw) = le_u8(i)?;
        let is_disabled = is_disabled_raw != 0;
        let (i, env_vis_raw) = le_u8(i)?;
        let environment_visibility = EnvironmentVisibility::from_bits_truncate(env_vis_raw);
        let (i, vertex_count) = le_u32(i)?;
        let (i, index_count) = le_u32(i)?;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut buckets = Vec::new();
        let mut face_visibility_flags = Vec::new();

        let mut current_i = i;
        if !is_disabled {
            let (i_next, v) = count(parse_vec3, vertex_count as usize).parse(current_i)?;
            vertices = v;
            let (i_next, idx) = count(le_u16, index_count as usize).parse(i_next)?;
            indices = idx;
            let (i_next, b) = count(
                GeometryBucket::parse,
                (buckets_per_side as u32).pow(2) as usize,
            )
            .parse(i_next)?;
            buckets = b;
            current_i = i_next;

            if environment_visibility != EnvironmentVisibility::NoLayer {
                let (i_next, f) = count(le_u8, (index_count / 3) as usize).parse(current_i)?;
                face_visibility_flags = f;
                current_i = i_next;
            }
        }

        Ok((
            current_i,
            SceneGraph {
                visibility_controller_path_hash,
                min_x,
                min_z,
                max_x,
                max_z,
                max_stick_out_x,
                max_stick_out_z,
                bucket_size_x,
                bucket_size_z,
                buckets_per_side,
                is_disabled,
                environment_visibility,
                vertex_count,
                index_count,
                vertices,
                indices,
                buckets,
                face_visibility_flags,
            },
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryBucket {
    pub max_stick_out_x: f32,
    pub max_stick_out_z: f32,
    pub start_index: u32,
    pub base_vertex: u32,
    pub inside_face_count: u16,
    pub sticking_out_face_count: u16,
}

impl GeometryBucket {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, max_stick_out_x) = le_f32(input)?;
        let (i, max_stick_out_z) = le_f32(i)?;
        let (i, start_index) = le_u32(i)?;
        let (i, base_vertex) = le_u32(i)?;
        let (i, inside_face_count) = le_u16(i)?;
        let (i, sticking_out_face_count) = le_u16(i)?;
        Ok((
            i,
            GeometryBucket {
                max_stick_out_x,
                max_stick_out_z,
                start_index,
                base_vertex,
                inside_face_count,
                sticking_out_face_count,
            },
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanarReflector {
    pub transform: Mat4,
    pub bounds: BoundingBox,
    pub normal: Vec3,
}

impl PlanarReflector {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, transform_raw) = count(le_f32, 16).parse(input)?;
        let transform = Mat4::from_cols_array(transform_raw.as_slice().try_into().unwrap());
        let (i, bounds) = BoundingBox::parse(i)?;
        let (i, normal_raw) = count(le_f32, 3).parse(i)?;
        let normal = Vec3::from_slice(&normal_raw);
        Ok((
            i,
            PlanarReflector {
                transform,
                bounds,
                normal,
            },
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderTextureOverride {
    pub id: u32,
    pub path: SizedStringU32,
}

impl ShaderTextureOverride {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, id) = le_u32(input)?;
        let (i, path) = SizedStringU32::parse(i)?;
        Ok((i, ShaderTextureOverride { id, path }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexDeclaration {
    pub usage: u32,
    pub elements: Vec<VertexElement>,
}

impl VertexDeclaration {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, usage) = le_u32(input)?;
        let (i, element_count) = le_u32(i)?;
        let (i, elements_raw) = count(VertexElement::parse, 15).parse(i)?;
        let elements = elements_raw[..element_count as usize].to_vec();
        Ok((i, VertexDeclaration { usage, elements }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexElement {
    pub name: ElementName,
    pub format: ElementFormat,
}

impl VertexElement {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, name_raw) = le_u32(input)?;
        let (i, format_raw) = le_u32(i)?;
        Ok((
            i,
            VertexElement {
                name: ElementName::from(name_raw),
                format: ElementFormat::from(format_raw),
            },
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexBuffer {
    pub environment_visibility: EnvironmentVisibility,
    pub buffer: Vec<u8>,
}

impl VertexBuffer {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, env_vis_raw) = le_u8(input)?;
        let environment_visibility = EnvironmentVisibility::from_bits_truncate(env_vis_raw);
        let (i, buffer_count) = le_u32(i)?;
        let (i, buffer) = take(buffer_count as usize)(i)?;
        Ok((
            i,
            VertexBuffer {
                environment_visibility,
                buffer: buffer.to_vec(),
            },
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexBuffer {
    pub environment_visibility: EnvironmentVisibility,
    pub buffer: Vec<u16>,
}

impl IndexBuffer {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, env_vis_raw) = le_u8(input)?;
        let environment_visibility = EnvironmentVisibility::from_bits_truncate(env_vis_raw);
        let (i, buffer_count) = le_u32(i)?;
        let (i, buffer) = count(le_u16, (buffer_count / 2) as usize).parse(i)?;
        Ok((
            i,
            IndexBuffer {
                environment_visibility,
                buffer,
            },
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum ElementFormat {
    Unknown = -1,
    XFloat32 = 0,
    XyFloat32 = 1,
    XyzFloat32 = 2,
    XyzwFloat32 = 3,
    BgraPacked8888 = 4,
    ZyxwPacked8888 = 5,
    RgbaPacked8888 = 6,
    XyPacked1616 = 7,
    XyzPacked161616 = 8,
    XyzwPacked16161616 = 9,
    XyPacked88 = 10,
    XyzPacked888 = 11,
    XyzwPacked8888 = 12,
}

impl From<u32> for ElementFormat {
    fn from(v: u32) -> Self {
        match v {
            0 => ElementFormat::XFloat32,
            1 => ElementFormat::XyFloat32,
            2 => ElementFormat::XyzFloat32,
            3 => ElementFormat::XyzwFloat32,
            4 => ElementFormat::BgraPacked8888,
            5 => ElementFormat::ZyxwPacked8888,
            6 => ElementFormat::RgbaPacked8888,
            7 => ElementFormat::XyPacked1616,
            8 => ElementFormat::XyzPacked161616,
            9 => ElementFormat::XyzwPacked16161616,
            10 => ElementFormat::XyPacked88,
            11 => ElementFormat::XyzPacked888,
            12 => ElementFormat::XyzwPacked8888,
            _ => ElementFormat::Unknown,
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum ElementName {
    Unknown = -1,
    Position = 0,
    BlendWeight = 1,
    Normal = 2,
    FogCoordinate = 3,
    PrimaryColor = 4,
    SecondaryColor = 5,
    BlendIndex = 6,
    Texcoord0 = 7,
    Texcoord1 = 8,
    Texcoord2 = 9,
    Texcoord3 = 10,
    Texcoord4 = 11,
    Texcoord5 = 12,
    Texcoord6 = 13,
    Texcoord7 = 14,
    Tangent = 15,
}

impl From<u32> for ElementName {
    fn from(v: u32) -> Self {
        match v {
            0 => ElementName::Position,
            1 => ElementName::BlendWeight,
            2 => ElementName::Normal,
            3 => ElementName::FogCoordinate,
            4 => ElementName::PrimaryColor,
            5 => ElementName::SecondaryColor,
            6 => ElementName::BlendIndex,
            7 => ElementName::Texcoord0,
            8 => ElementName::Texcoord1,
            9 => ElementName::Texcoord2,
            10 => ElementName::Texcoord3,
            11 => ElementName::Texcoord4,
            12 => ElementName::Texcoord5,
            13 => ElementName::Texcoord6,
            14 => ElementName::Texcoord7,
            15 => ElementName::Tangent,
            _ => ElementName::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeagueMapGeoMesh {
    pub vertex_count: u32,
    pub vertex_declaration_count: u32,
    pub vertex_declaration_index_base: u32,
    pub vertex_buffer_indexes: Vec<u32>,
    pub index_count: u32,
    pub index_buffer_id: u32,
    pub environment_visibility: EnvironmentVisibility,
    pub visibility_controller_path_hash: u32,
    pub submeshes: Vec<Submesh>,
    pub disable_backface_culling: bool,
    pub bounding_box: BoundingBox,
    pub transform: Vec<f32>,
    pub quality_filter: QualityFilter,
    pub layer_transition_behavior: LayerTransitionBehavior,
    pub render_flags: u16,
    pub baked_light: Channel,
    pub stationary_light: Channel,
    pub texture_overrides: Vec<TextureOverride>,
    pub baked_paint_scale: Vec2,
    pub baked_paint_bias: Vec2,
}

impl LeagueMapGeoMesh {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, vertex_count) = le_u32(input)?;
        let (i, vertex_declaration_count) = le_u32(i)?;
        let (i, vertex_declaration_index_base) = le_u32(i)?;
        let (i, vertex_buffer_indexes) =
            count(le_u32, vertex_declaration_count as usize).parse(i)?;
        let (i, index_count) = le_u32(i)?;
        let (i, index_buffer_id) = le_u32(i)?;
        let (i, env_vis_raw) = le_u8(i)?;
        let environment_visibility = EnvironmentVisibility::from_bits_truncate(env_vis_raw);
        let (i, visibility_controller_path_hash) = le_u32(i)?;
        let (i, submesh_count) = le_u32(i)?;
        let (i, submeshes) = count(Submesh::parse, submesh_count as usize).parse(i)?;
        let (i, disable_backface_culling_raw) = le_u8(i)?;
        let disable_backface_culling = disable_backface_culling_raw != 0;
        let (i, bounding_box) = BoundingBox::parse(i)?;
        let (i, transform) = count(le_f32, 16).parse(i)?;
        let (i, quality_filter_raw) = le_u8(i)?;
        let quality_filter = QualityFilter::from(quality_filter_raw);
        let (i, layer_transition_behavior_raw) = le_u8(i)?;
        let layer_transition_behavior =
            LayerTransitionBehavior::from(layer_transition_behavior_raw);
        let (i, render_flags) = le_u16(i)?;
        let (i, baked_light) = Channel::parse(i)?;
        let (i, stationary_light) = Channel::parse(i)?;
        let (i, texture_override_count) = le_u32(i)?;
        let (i, texture_overrides) =
            count(TextureOverride::parse, texture_override_count as usize).parse(i)?;
        let (i, baked_paint_scale_raw) = count(le_f32, 2).parse(i)?;
        let baked_paint_scale = Vec2::from_slice(&baked_paint_scale_raw);
        let (i, baked_paint_bias_raw) = count(le_f32, 2).parse(i)?;
        let baked_paint_bias = Vec2::from_slice(&baked_paint_bias_raw);

        Ok((
            i,
            LeagueMapGeoMesh {
                vertex_count,
                vertex_declaration_count,
                vertex_declaration_index_base,
                vertex_buffer_indexes,
                index_count,
                index_buffer_id,
                environment_visibility,
                visibility_controller_path_hash,
                submeshes,
                disable_backface_culling,
                bounding_box,
                transform,
                quality_filter,
                layer_transition_behavior,
                render_flags,
                baked_light,
                stationary_light,
                texture_overrides,
                baked_paint_scale,
                baked_paint_bias,
            },
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submesh {
    pub hash: u32,
    pub material_name: SizedStringU32,
    pub start_index: u32,
    pub submesh_index_count: u32,
    pub min_vertex: u32,
    pub max_vertex: u32,
}

impl Submesh {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, hash) = le_u32(input)?;
        let (i, material_name) = SizedStringU32::parse(i)?;
        let (i, start_index) = le_u32(i)?;
        let (i, submesh_index_count) = le_u32(i)?;
        let (i, min_vertex) = le_u32(i)?;
        let (i, max_vertex) = le_u32(i)?;
        Ok((
            i,
            Submesh {
                hash,
                material_name,
                start_index,
                submesh_index_count,
                min_vertex,
                max_vertex,
            },
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub texture: SizedStringU32,
    pub uv_scale: Vec2,
    pub uv_offset: Vec2,
}

impl Channel {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, texture) = SizedStringU32::parse(input)?;
        let (i, uv_scale_raw) = count(le_f32, 2).parse(i)?;
        let uv_scale = Vec2::from_slice(&uv_scale_raw);
        let (i, uv_offset_raw) = count(le_f32, 2).parse(i)?;
        let uv_offset = Vec2::from_slice(&uv_offset_raw);
        Ok((
            i,
            Channel {
                texture,
                uv_scale,
                uv_offset,
            },
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureOverride {
    pub sampler_id: u32,
    pub texture_path: SizedStringU32,
}

impl TextureOverride {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, sampler_id) = le_u32(input)?;
        let (i, texture_path) = SizedStringU32::parse(i)?;
        Ok((
            i,
            TextureOverride {
                sampler_id,
                texture_path,
            },
        ))
    }
}

use crate::common::SizedStringU32;

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

fn parse_vec3(input: &[u8]) -> IResult<&[u8], Vec3> {
    let (i, x) = le_f32(input)?;
    let (i, y) = le_f32(i)?;
    let (i, z) = le_f32(i)?;
    Ok((i, Vec3::new(x, y, z)))
}
