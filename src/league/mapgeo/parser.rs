use crate::league::{
    BinMat4, BinVec3, BoundingBox, ElementFormat, ElementName, EnvironmentVisibility,
    LeagueMapGeoMesh, SizedString,
};
use binrw::binread;

#[binread]
#[derive(Debug)]
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
#[derive(Debug)]
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
    pub environment_visibility: EnvironmentVisibility,

    pub vertex_count: u32,
    pub index_count: u32,

    #[br(if(!is_disabled))]
    #[br(count = vertex_count)]
    pub vertices: Vec<BinVec3>,

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
#[derive(Debug)]
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
#[derive(Debug)]
#[br(little)]
pub struct PlanarReflector {
    pub transform: BinMat4,
    pub bounds: BoundingBox,
    pub normal: BinVec3,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct ShaderTextureOverride {
    pub id: u32,
    pub path: SizedString,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct VertexDeclaration {
    pub usage: u32,

    pub element_count: u32,
    #[br(count = 15, map = |v: Vec<VertexElement>| (&v[..element_count as usize]).to_vec())]
    pub elements: Vec<VertexElement>,
}

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct VertexElement {
    pub name: ElementName,
    pub format: ElementFormat,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct VertexBuffer {
    pub environment_visibility: EnvironmentVisibility,

    pub buffer_count: u32,
    #[br(count = buffer_count)]
    pub buffer: Vec<u8>,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct IndexBuffer {
    pub environment_visibility: EnvironmentVisibility,

    pub buffer_count: u32,
    #[br(count = buffer_count / 2)]
    pub buffer: Vec<u16>,
}
