use std::collections::HashMap;

use bevy::{
    color::Color,
    math::{Mat4, Vec2, Vec3, Vec4},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MapPlaceableContainerItems {
    Unk0x42239bf8(Unk0x42239bf8),
    MapScriptLocator(MapScriptLocator),
    MapParticle(MapParticle),
    GdsMapObject(GdsMapObject),
    Unk0x111a9fcc(Unk0x111a9fcc),
    Unk0x7ad3dda(Unk0x7ad3dda),
    MapGroup(MapGroup),
    MapCubemapProbe(MapCubemapProbe),
    Unk0x0,
    MapLocator(MapLocator),
    Unk0xff6e8118(Unk0xff6e8118),
    Unk0xf4a21c35(Unk0xf4a21c35),
    Unk0x3c995caf(Unk0x3c995caf),
    Unk0xc71ee7fb(Unk0xc71ee7fb),
    Unk0x3c2bf0c0(Unk0x3c2bf0c0),
    MapAudio(MapAudio),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum VfxEmitterDefinitionDataPrimitive {
    VfxPrimitiveMesh(VfxPrimitiveMesh),
    VfxPrimitiveCameraTrail(VfxPrimitiveCameraTrail),
    VfxPrimitiveArbitraryTrail(VfxPrimitiveArbitraryTrail),
    VfxPrimitiveArbitraryQuad(VfxPrimitiveArbitraryQuad),
    Unk0x8df5fcf7(Unk0x8df5fcf7),
    VfxPrimitiveRay(VfxPrimitiveRay),
    VfxPrimitivePlanarProjection(VfxPrimitivePlanarProjection),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum VfxEmitterDefinitionDataSpawnShape {
    Unk0xee39916f(Unk0xee39916f),
    VfxShapeSphere(VfxShapeSphere),
    VfxShapeBox(VfxShapeBox),
    VfxShapeCylinder(VfxShapeCylinder),
    VfxShapeLegacy(VfxShapeLegacy),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TimedWaveBehaviorInfoBehavior {
    ConstantWaveBehavior(ConstantWaveBehavior),
    RotatingWaveBehavior(RotatingWaveBehavior),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BarracksMinionConfigWaveBehavior {
    InhibitorWaveBehavior(InhibitorWaveBehavior),
    ConstantWaveBehavior(ConstantWaveBehavior),
    TimedVariableWaveBehavior(TimedVariableWaveBehavior),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MapContainerComponents {
    Unk0xcdb1c8f6(Unk0xcdb1c8f6),
    MapBakeProperties(MapBakeProperties),
    MapSunProperties(MapSunProperties),
    MapTerrainPaint(MapTerrainPaint),
    MapNavGrid(MapNavGrid),
    Unk0xcf4a55da(Unk0xcf4a55da),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxProbabilityTableData {
    pub key_values: Option<Vec<f32>>,
    pub key_times: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapNavGrid {
    pub nav_grid_path: String,
    pub nav_grid_config: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveMesh {
    pub m_mesh: VfxMeshDefinitionData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x7faa90a0 {
    pub skin: String,
    pub idle_animation_name: String,
    pub character_record: String,
    pub team: u32,
    pub play_idle_animation: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x82cab1b3 {
    pub lane: u16,
    pub position: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValueVector3 {
    pub dynamics: Option<VfxAnimatedVector3fVariableData>,
    pub constant_value: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldCollectionDefinitionData {
    pub field_attraction_definitions: Option<Vec<VfxFieldAttractionDefinitionData>>,
    pub field_acceleration_definitions: Option<Vec<VfxFieldAccelerationDefinitionData>>,
    pub field_orbital_definitions: Option<Vec<VfxFieldOrbitalDefinitionData>>,
    pub field_drag_definitions: Option<Vec<VfxFieldDragDefinitionData>>,
    pub field_noise_definitions: Option<Vec<VfxFieldNoiseDefinitionData>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xee39916f {
    pub emit_offset: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxAlphaErosionDefinitionData {
    pub erosion_map_address_mode: Option<u8>,
    pub linger_erosion_drive_curve: Option<ValueFloat>,
    pub use_linger_erosion_drive_curve: Option<bool>,
    pub erosion_map_channel_mixer: Option<ValueColor>,
    pub erosion_drive_curve: Option<ValueFloat>,
    pub erosion_feather_out: Option<f32>,
    pub erosion_feather_in: Option<f32>,
    pub erosion_slice_width: Option<f32>,
    pub erosion_map_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimedVariableWaveBehavior {
    pub behaviors: Vec<TimedWaveBehaviorInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapScriptLocator {
    pub transform: Mat4,
    pub script_name: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChildMapVisibilityController {
    pub parents: Vec<u32>,
    pub path_hash: u32,
    pub parent_mode: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xe07edfa4 {
    pub path_hash: u32,
    pub name: u32,
    pub default_visible: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeSphere {
    pub radius: Option<f32>,
    pub flags: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedVector3fVariableData {
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub values: Vec<Vec3>,
    pub times: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxSoftParticleDefinitionData {
    pub begin_in: Option<f32>,
    pub begin_out: Option<f32>,
    pub delta_in: Option<f32>,
    pub delta_out: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxAssetRemap {
    pub new_asset: String,
    pub old_asset: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf4a21c35 {
    pub definition: Unk0xfcb92181,
    pub name: u32,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapGroup {
    pub name: String,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapSunProperties {
    pub sky_light_color: Vec4,
    pub ground_color: Vec4,
    pub fog_alternate_color: Vec4,
    pub sun_direction: Vec3,
    pub fog_color: Vec4,
    pub fog_start_and_end: Vec2,
    pub light_map_color_scale: f32,
    pub horizon_color: Vec4,
    pub sky_light_scale: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxPaletteDefinitionData {
    pub palette_count: Option<i32>,
    pub pallete_src_mix_color: Option<ValueColor>,
    pub palette_texture: Option<String>,
    pub palette_texture_address_mode: Option<u8>,
    pub palette_selector: Option<ValueVector3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialShaderSamplerDef {
    pub texture_path: String,
    pub address_w: Option<u32>,
    pub address_v: Option<u32>,
    pub address_u: Option<u32>,
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConstantWaveBehavior {
    pub spawn_count: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapPlaceableContainer {
    pub items: HashMap<u32, MapPlaceableContainerItems>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapNavGridOverlays {
    pub overlays: Vec<MapNavGridOverlay>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x8df5fcf7 {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9d9f60d2 {
    pub unk_0xf1d3a034: Option<bool>,
    pub character_record: String,
    pub skin: String,
    pub r#type: Option<u16>,
    pub unk_0xdbde2288: Option<Vec<Unk0x82cab1b3>>,
    pub unk_0x397fe037: Option<bool>,
    pub team: Option<u32>,
    pub unk_0xde46f1d8: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapNavGridOverlay {
    pub nav_grid_file_name: String,
    pub regions_filename: String,
    pub name: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueVector2 {
    pub dynamics: VfxAnimatedVector2fVariableData,
    pub constant_value: Option<Vec2>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RotatingWaveBehavior {
    pub spawn_counts_by_wave: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveArbitraryQuad {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeCylinder {
    pub height: Option<f32>,
    pub flags: Option<u8>,
    pub radius: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EsportsBannerData {
    pub banner_name: String,
    pub team: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxTrailDefinitionData {
    pub m_smoothing_mode: Option<u8>,
    pub m_birth_tiling_size: ValueVector3,
    pub m_cutoff: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xc406a533 {
    pub path_hash: u32,
    pub unk_0x27639032: u8,
    pub name: u32,
    pub default_visible: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterDefinitionData {
    pub primitive: Option<VfxEmitterDefinitionDataPrimitive>,
    pub birth_acceleration: Option<ValueVector3>,
    pub color_look_up_offsets: Option<Vec2>,
    pub texture: Option<String>,
    pub depth_bias_factors: Option<Vec2>,
    pub child_particle_set_definition: Option<VfxChildParticleSetDefinitionData>,
    pub color_look_up_type_x: Option<u8>,
    pub birth_uv_scroll_rate: Option<ValueVector2>,
    pub particle_linger_type: Option<u8>,
    pub emitter_uv_scroll_rate: Option<Vec2>,
    pub rate_by_velocity_function: Option<ValueVector2>,
    pub importance: Option<u8>,
    pub emitter_linger: Option<f32>,
    pub post_rotate_orientation_axis: Option<Vec3>,
    pub emission_surface_definition: Option<VfxEmissionSurfaceData>,
    pub is_emitter_space: Option<bool>,
    pub birth_rotation0: Option<ValueVector3>,
    pub direction_velocity_scale: Option<f32>,
    pub emitter_name: String,
    pub tex_address_mode_base: Option<u8>,
    pub world_acceleration: Option<IntegratedValueVector3>,
    pub birth_drag: Option<ValueVector3>,
    pub scale0: Option<ValueVector3>,
    pub particle_color_texture: Option<String>,
    pub field_collection_definition: Option<VfxFieldCollectionDefinitionData>,
    pub disabled: Option<bool>,
    pub texture_flip_u: Option<bool>,
    pub mesh_render_flags: Option<u8>,
    pub linger: Option<VfxLingerDefinitionData>,
    pub birth_frame_rate: Option<ValueFloat>,
    pub emission_mesh_name: Option<String>,
    pub alpha_erosion_definition: Option<VfxAlphaErosionDefinitionData>,
    pub is_following_terrain: Option<bool>,
    pub legacy_simple: Option<VfxEmitterLegacySimple>,
    pub use_navmesh_mask: Option<bool>,
    pub birth_uv_offset: Option<ValueVector2>,
    pub rotation_override: Option<Vec3>,
    pub particle_linger: Option<f32>,
    pub birth_velocity: Option<ValueVector3>,
    pub bind_weight: Option<ValueFloat>,
    pub modulation_factor: Option<Vec4>,
    pub direction_velocity_min_scale: Option<f32>,
    pub time_active_during_period: Option<f32>,
    pub has_post_rotate_orientation: Option<bool>,
    pub alpha_ref: Option<u8>,
    pub emitter_position: Option<ValueVector3>,
    pub rate: Option<ValueFloat>,
    pub distortion_definition: Option<VfxDistortionDefinitionData>,
    pub rotation0: Option<IntegratedValueVector3>,
    pub is_uniform_scale: Option<bool>,
    pub velocity: Option<ValueVector3>,
    pub period: Option<f32>,
    pub color_look_up_scales: Option<Vec2>,
    pub color: Option<ValueColor>,
    pub particle_lifetime: Option<ValueFloat>,
    pub particle_uv_rotate_rate: Option<IntegratedValueFloat>,
    pub is_direction_oriented: Option<bool>,
    pub reflection_definition: Option<VfxReflectionDefinitionData>,
    pub disable_backface_cull: Option<bool>,
    pub lifetime: Option<f32>,
    pub soft_particle_params: Option<VfxSoftParticleDefinitionData>,
    pub filtering: Option<VfxEmitterFiltering>,
    pub does_cast_shadow: Option<bool>,
    pub texture_mult: Option<VfxTextureMultDefinitionData>,
    pub uv_rotation: Option<ValueFloat>,
    pub has_variable_start_time: Option<bool>,
    pub particle_uv_scroll_rate: Option<IntegratedValueVector2>,
    pub maximum_rate_by_velocity: Option<f32>,
    pub is_local_orientation: Option<bool>,
    pub falloff_texture: Option<String>,
    pub birth_uv_rotate_rate: Option<ValueFloat>,
    pub translation_override: Option<Vec3>,
    pub uv_scale: Option<ValueVector2>,
    pub birth_scale0: Option<ValueVector3>,
    pub birth_rotational_velocity0: Option<ValueVector3>,
    pub particle_is_local_orientation: Option<bool>,
    pub is_rotation_enabled: Option<bool>,
    pub time_before_first_emission: Option<f32>,
    pub chance_to_not_exist: Option<f32>,
    pub drag: Option<ValueVector3>,
    pub uv_scroll_clamp: Option<bool>,
    pub birth_orbital_velocity: Option<ValueVector3>,
    pub unk_0xcb13aff1: Option<f32>,
    pub is_random_start_frame: Option<bool>,
    pub is_single_particle: Option<bool>,
    pub misc_render_flags: Option<u8>,
    pub is_ground_layer: Option<bool>,
    pub flex_shape_definition: Option<VfxFlexShapeDefinitionData>,
    pub num_frames: Option<u16>,
    pub stencil_ref: Option<u8>,
    pub use_emission_mesh_normal_for_birth: Option<bool>,
    pub blend_mode: Option<u8>,
    pub palette_definition: Option<VfxPaletteDefinitionData>,
    pub frame_rate: Option<f32>,
    pub start_frame: Option<u16>,
    pub acceleration: Option<ValueVector3>,
    pub birth_color: Option<ValueColor>,
    pub pass: Option<i16>,
    pub tex_div: Option<Vec2>,
    pub spawn_shape: Option<VfxEmitterDefinitionDataSpawnShape>,
    pub color_look_up_type_y: Option<u8>,
    pub uv_mode: Option<u8>,
    pub emission_mesh_scale: Option<f32>,
    pub censor_modulate_value: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueFloat {
    pub constant_value: Option<f32>,
    pub dynamics: VfxAnimatedFloatVariableData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x42239bf8 {
    pub definition: Unk0x429a2180,
    pub transform: Mat4,
    pub name: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueVector3 {
    pub constant_value: Option<Vec3>,
    pub dynamics: VfxAnimatedVector3fVariableData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxProjectionDefinitionData {
    pub color_modulate: ValueColor,
    pub m_y_range: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxChildParticleSetDefinitionData {
    pub children_identifiers: Vec<VfxChildIdentifier>,
    pub children_probability: Option<ValueFloat>,
    pub bone_to_spawn_at: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxDistortionDefinitionData {
    pub distortion: f32,
    pub distortion_mode: Option<u8>,
    pub normal_map_texture: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf775806c {
    pub unk_0xd1318f26: f32,
    pub character_record: String,
    pub team: Option<u32>,
    pub unk_0x651de225: f32,
    pub skin: String,
    pub unk_0xf908963: Vec3,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x3c2bf0c0 {
    pub unk_0xbbe68da1: Option<bool>,
    pub definition: Unk0x9d9f60d2,
    pub name: u32,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterFiltering {
    pub spectator_policy: Option<u8>,
    pub keywords_excluded: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xd82714cc {
    pub name: u32,
    pub color: Option<Color>,
    pub flags: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapBakeProperties {
    pub light_grid_size: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldDragDefinitionData {
    pub strength: ValueFloat,
    pub position: Option<ValueVector3>,
    pub radius: ValueFloat,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xcdb1c8f6 {
    pub unk_0x6355dd6f: Vec<Unk0x6355dd6f>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialPassDef {
    pub shader: u32,
    pub dst_alpha_blend_factor: Option<u32>,
    pub write_mask: Option<u32>,
    pub dst_color_blend_factor: Option<u32>,
    pub src_alpha_blend_factor: Option<u32>,
    pub blend_enable: Option<bool>,
    pub cull_enable: Option<bool>,
    pub src_color_blend_factor: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NavGridTerrainConfig {
    pub tags: Vec<Unk0xd82714cc>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxSystemDefinitionData {
    pub simple_emitter_definition_data: Option<Vec<VfxEmitterDefinitionData>>,
    pub build_up_time: Option<f32>,
    pub particle_path: String,
    pub flags: Option<u16>,
    pub complex_emitter_definition_data: Option<Vec<VfxEmitterDefinitionData>>,
    pub sound_persistent_default: Option<String>,
    pub sound_on_create_default: Option<String>,
    pub override_scale_cap: Option<f32>,
    pub transform: Option<Mat4>,
    pub visibility_radius: Option<f32>,
    pub asset_remapping_table: Option<Vec<VfxAssetRemap>>,
    pub particle_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxLingerDefinitionData {
    pub use_separate_linger_color: Option<bool>,
    pub separate_linger_color: Option<ValueColor>,
    pub linger_scale: Option<ValueVector3>,
    pub use_linger_scale: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveCameraTrail {
    pub m_trail: VfxTrailDefinitionData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldOrbitalDefinitionData {
    pub is_local_space: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InhibitorWaveBehavior {
    pub spawn_count_per_inhibitor_down: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveRay {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x2bfb084c {
    pub group_name: String,
    pub unk_0xec01928c: Option<bool>,
    pub tags: Vec<Unk0xf6f4bb5f>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeBox {
    pub flags: Option<u8>,
    pub size: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xff6e8118 {
    pub definition: Unk0x7faa90a0,
    pub name: u32,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapTerrainPaint {
    pub terrain_paint_texture_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveArbitraryTrail {
    pub m_trail: VfxTrailDefinitionData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GdsMapObject {
    pub transform: Mat4,
    pub name: String,
    pub box_max: Option<Vec3>,
    pub r#type: u8,
    pub extra_info: Option<Vec<GdsMapObjectBannerInfo>>,
    pub visibility_controller: Option<u32>,
    pub box_min: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x111a9fcc {
    pub definition: Unk0xf775806c,
    pub transform: Mat4,
    pub name: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialSwitchDef {
    pub name: String,
    pub on: Option<bool>,
    pub group: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x7ad3dda {
    pub definition: Unk0x8ad25772,
    pub unk_0xbbe68da1: bool,
    pub name: u32,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValueVector2 {
    pub dynamics: Option<VfxAnimatedVector2fVariableData>,
    pub constant_value: Option<Vec2>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxChildIdentifier {
    pub effect: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GdsMapObjectBannerInfo {
    pub banner_data: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimedWaveBehaviorInfo {
    pub start_time_secs: i32,
    pub behavior: TimedWaveBehaviorInfoBehavior,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialChildTechniqueDef {
    pub shader_macros: HashMap<String, String>,
    pub name: String,
    pub parent_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapLocator {
    pub transform: Mat4,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xfde6a2d7 {
    pub barracks_config: u32,
    pub team: Option<u32>,
    pub unk_0xdb6ea1a7: Option<u32>,
    pub unk_0xdbde2288: Vec<Unk0x82cab1b3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldAccelerationDefinitionData {
    pub acceleration: ValueVector3,
    pub is_local_space: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MutatorMapVisibilityController {
    pub path_hash: u32,
    pub mutator_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x6355dd6f {
    pub visibility_controller: u32,
    pub chunk: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeLegacy {
    pub emit_rotation_angles: Option<Vec<ValueFloat>>,
    pub emit_rotation_axes: Option<Vec<Vec3>>,
    pub emit_offset: Option<ValueVector3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapParticle {
    pub transitional: Option<bool>,
    pub name: String,
    pub group_name: Option<String>,
    pub m_visibility_flags: Option<u8>,
    pub visibility_mode: Option<u32>,
    pub eye_candy: Option<bool>,
    pub transform: Mat4,
    pub color_modulate: Option<Vec4>,
    pub system: u32,
    pub visibility_controller: Option<u32>,
    pub start_disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValueFloat {
    pub dynamics: Option<VfxAnimatedFloatVariableData>,
    pub constant_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmissionSurfaceData {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialDef {
    pub sampler_values: Vec<StaticMaterialShaderSamplerDef>,
    pub r#type: u32,
    pub param_values: Vec<StaticMaterialShaderParamDef>,
    pub shader_macros: HashMap<String, String>,
    pub techniques: Vec<StaticMaterialTechniqueDef>,
    pub name: String,
    pub switches: Option<Vec<StaticMaterialSwitchDef>>,
    pub child_techniques: Option<Vec<StaticMaterialChildTechniqueDef>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedColorVariableData {
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub values: Vec<Vec4>,
    pub times: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxTextureMultDefinitionData {
    pub tex_address_mode_mult: Option<u8>,
    pub birth_uv_offset_mult: Option<ValueVector2>,
    pub texture_mult_filp_v: Option<bool>,
    pub uv_transform_center_mult: Option<Vec2>,
    pub uv_scroll_alpha_mult: Option<bool>,
    pub particle_integrated_uv_scroll_mult: Option<IntegratedValueVector2>,
    pub texture_mult: Option<String>,
    pub birth_uv_rotate_rate_mult: Option<ValueFloat>,
    pub tex_div_mult: Option<Vec2>,
    pub particle_integrated_uv_rotate_mult: Option<IntegratedValueFloat>,
    pub uv_scroll_clamp_mult: Option<bool>,
    pub uv_scale_mult: Option<ValueVector2>,
    pub uv_rotation_mult: Option<ValueFloat>,
    pub birth_uv_scroll_rate_mult: Option<ValueVector2>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapAudio {
    pub transform: Mat4,
    pub name: String,
    pub event_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MinionUpgradeConfig {
    pub hp_max_bonus: f32,
    pub damage_upgrade_late: Option<f32>,
    pub armor_max: Option<f32>,
    pub gold_max: Option<f32>,
    pub damage_max: f32,
    pub armor_upgrade_growth: Option<f32>,
    pub hp_upgrade_late: Option<f32>,
    pub hp_upgrade: f32,
    pub unk_0x726ae049: Option<f32>,
    pub damage_upgrade: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xfcb92181 {
    pub team: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BarracksMinionConfig {
    pub minion_type: u8,
    pub wave_behavior: BarracksMinionConfigWaveBehavior,
    pub unk_0x8a3fc6eb: u32,
    pub minion_upgrade_stats: MinionUpgradeConfig,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x8ad25772 {
    pub system: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValueColor {
    pub constant_value: Option<Vec4>,
    pub dynamics: Option<VfxAnimatedColorVariableData>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BarracksConfig {
    pub minion_spawn_interval_secs: f32,
    pub initial_spawn_time_secs: f32,
    pub upgrades_before_late_game_scaling: i32,
    pub move_speed_increase_interval_secs: f32,
    pub exp_radius: f32,
    pub move_speed_increase_increment: i32,
    pub upgrade_interval_secs: f32,
    pub move_speed_increase_max_times: i32,
    pub gold_radius: f32,
    pub wave_spawn_interval_secs: f32,
    pub move_speed_increase_initial_delay_secs: f32,
    pub units: Vec<BarracksMinionConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapContainer {
    pub components: Vec<MapContainerComponents>,
    pub bounds_max: Vec2,
    pub lowest_walkable_height: f32,
    pub mesh_combine_radius: f32,
    pub chunks: HashMap<u32, u32>,
    pub convert_streams_to_half_float: bool,
    pub map_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialShaderParamDef {
    pub value: Option<Vec4>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitivePlanarProjection {
    pub m_projection: VfxProjectionDefinitionData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldAttractionDefinitionData {
    pub position: ValueVector3,
    pub acceleration: ValueFloat,
    pub radius: ValueFloat,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xcf4a55da {
    pub overlays: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x3c995caf {
    pub transform: Mat4,
    pub name: String,
    pub segments: Vec<Vec3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxReflectionDefinitionData {
    pub reflection_map_texture: Option<String>,
    pub reflection_fresnel_color: Option<Vec4>,
    pub fresnel_color: Option<Vec4>,
    pub reflection_opacity_glancing: Option<f32>,
    pub reflection_opacity_direct: Option<f32>,
    pub fresnel: Option<f32>,
    pub reflection_fresnel: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxFlexShapeDefinitionData {
    pub scale_birth_scale_by_bound_object_size: Option<f32>,
    pub scale_emit_offset_by_bound_object_height: Option<f32>,
    pub scale_emit_offset_by_bound_object_size: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldNoiseDefinitionData {
    pub velocity_delta: ValueFloat,
    pub frequency: ValueFloat,
    pub axis_fraction: Vec3,
    pub radius: ValueFloat,
    pub position: Option<ValueVector3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterLegacySimple {
    pub birth_scale: ValueFloat,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf6f4bb5f {
    pub name: String,
    pub color: Option<Color>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x429a2180 {
    pub camp_name: String,
    pub scoreboard_timer: Option<u16>,
    pub minimap_icon_offset: Option<Vec3>,
    pub team: u32,
    pub unk_0x7d27af7f: Option<bool>,
    pub reveal_event: Option<u16>,
    pub tags: Option<Vec<u32>>,
    pub camp_level: Option<u16>,
    pub stop_spawn_time_secs: Option<f32>,
    pub minimap_icon: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xc71ee7fb {
    pub name: u32,
    pub transform: Mat4,
    pub definition: Unk0xfde6a2d7,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxMeshDefinitionData {
    pub m_simple_mesh_name: Option<String>,
    pub m_lock_mesh_to_attachment: Option<bool>,
    pub m_mesh_name: Option<String>,
    pub m_animation_name: Option<String>,
    pub m_mesh_skeleton_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapCubemapProbe {
    pub name: String,
    pub transform: Mat4,
    pub cubemap_probe_scale: f32,
    pub cubemap_probe_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedVector2fVariableData {
    pub values: Vec<Vec2>,
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub times: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedFloatVariableData {
    pub values: Vec<f32>,
    pub times: Vec<f32>,
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialTechniqueDef {
    pub passes: Vec<StaticMaterialPassDef>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NavGridConfig {
    pub terrain_config: u32,
    pub region_groups: Vec<Unk0x2bfb084c>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xec733fe2 {
    pub unk_0x8bff8cdf: u8,
    pub name: u32,
    pub path_hash: u32,
    pub default_visible: Option<bool>,
}
