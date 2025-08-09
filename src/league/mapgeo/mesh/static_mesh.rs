use crate::league::{BoundingBox, EnvironmentVisibility, Vector2};
use binrw::binread;

use super::enums::{parse_layer_transition_behavior, parse_quality_filter, LayerTransitionBehavior, QualityFilter};
use super::types::{Channel, Submesh, TextureOverride};

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
