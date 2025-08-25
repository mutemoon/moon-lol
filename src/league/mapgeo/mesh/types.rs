use crate::league::{SizedStringU32, Vector2};
use binrw::binread;

#[binread]
#[derive(Debug)]
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
#[derive(Debug, Clone)]
#[br(little)]
pub struct Channel {
    pub texture: SizedStringU32,
    pub uv_scale: Vector2,
    pub uv_offset: Vector2,
}

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct TextureOverride {
    pub sampler_id: u32,
    pub texture_path: SizedStringU32,
}
