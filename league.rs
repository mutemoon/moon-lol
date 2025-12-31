#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbilityResourceByCoefficientCalculationPart {
    pub m_ability_resource: Option<u8>,
    pub m_coefficient: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AcceleratingMovement {
    pub m_acceleration: f32,
    pub m_initial_speed: f32,
    pub m_max_speed: f32,
    pub m_min_speed: f32,
    pub m_offset_initial_target_height: f32,
    pub m_project_target_to_cast_range: bool,
    pub m_start_bone_name: String,
    pub m_target_height_augment: f32,
    pub m_tracks_target: bool,
    pub m_use_height_offset_at_end: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnchorDouble {
    pub anchor_left: Option<Vec2>,
    pub anchor_right: Option<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnchorSingle {
    pub anchor: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtlasData {
    pub m_texture_name: String,
    pub m_texture_source_resolution_height: Option<u32>,
    pub m_texture_source_resolution_width: Option<u32>,
    pub m_texture_uv: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtlasData3SliceH {
    pub left_right_widths: Vec2,
    pub m_texture_name: String,
    pub m_texture_source_resolution_height: u32,
    pub m_texture_source_resolution_width: u32,
    pub texture_us: Vec4,
    pub texture_vs: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtlasData3SliceV {
    pub m_texture_name: String,
    pub m_texture_source_resolution_height: u32,
    pub m_texture_source_resolution_width: u32,
    pub texture_us: Vec2,
    pub texture_vs: Vec4,
    pub top_bottom_heights: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtlasData9Slice {
    pub left_right_widths: Vec2,
    pub m_texture_name: String,
    pub m_texture_source_resolution_height: u32,
    pub m_texture_source_resolution_width: u32,
    pub texture_us: Vec4,
    pub texture_vs: Vec4,
    pub top_bottom_heights: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuffCounterByCoefficientCalculationPart {
    pub m_buff_name: u32,
    pub m_coefficient: f32,
    pub m_icon_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuffCounterByNamedDataValueCalculationPart {
    pub m_buff_name: u32,
    pub m_data_value: u32,
    pub m_icon_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuffData {
    pub m_description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DrawablePositionLocator {
    pub angle_offset_radian: Option<f32>,
    pub base_position: Option<u32>,
    pub distance_offset: Option<f32>,
    pub orientation_type: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicMaterialDef {
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumAbilityResourceByCoefficientCalculationPart {
    AbilityResourceByCoefficientCalculationPart(AbilityResourceByCoefficientCalculationPart),
    BuffCounterByCoefficientCalculationPart(BuffCounterByCoefficientCalculationPart),
    BuffCounterByNamedDataValueCalculationPart(BuffCounterByNamedDataValueCalculationPart),
    ExponentSubPartsCalculationPart(ExponentSubPartsCalculationPart),
    NamedDataValueCalculationPart(NamedDataValueCalculationPart),
    NumberCalculationPart(NumberCalculationPart),
    ProductOfSubPartsCalculationPart(ProductOfSubPartsCalculationPart),
    StatByCoefficientCalculationPart(StatByCoefficientCalculationPart),
    StatByNamedDataValueCalculationPart(StatByNamedDataValueCalculationPart),
    StatBySubPartCalculationPart(StatBySubPartCalculationPart),
    StatEfficiencyPerHundred(StatEfficiencyPerHundred),
    SubPartScaledProportionalToStat(SubPartScaledProportionalToStat),
    SumOfSubPartsCalculationPart(SumOfSubPartsCalculationPart),
    Unk0x382277da(Unk0x382277da),
    Unk0x8a96ea3c(Unk0x8a96ea3c),
    Unk0xf3cbe7b2(Unk0xf3cbe7b2),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumAnchor {
    AnchorDouble(AnchorDouble),
    AnchorSingle(AnchorSingle),
    Unk0xf090d2e7(Unk0xf090d2e7),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumCastOnHit {
    CastOnHit,
    CastOnMovementComplete,
    DestroyOnExitMap,
    DestroyOnHit,
    DestroyOnMovementComplete,
    Unk0x91fd0920,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumData {
    AtlasData(AtlasData),
    AtlasData3SliceH(AtlasData3SliceH),
    AtlasData3SliceV(AtlasData3SliceV),
    AtlasData9Slice(AtlasData9Slice),
    LooseUiTextureData(LooseUiTextureData),
    LooseUiTextureData3SliceH(LooseUiTextureData3SliceH),
    LooseUiTextureData3SliceV(LooseUiTextureData3SliceV),
    LooseUiTextureData9Slice(LooseUiTextureData9Slice),
    Unk0x5eaead1a,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumDirection {
    Direction,
    Location,
    MySelf,
    SelfAoe,
    TargetOrLocation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumFacing {
    VeritcalFacingMatchVelocity,
    VerticalFacingFaceTarget,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumHeightSolver {
    BlendedLinearHeightSolver,
    CurveTheDifferenceHeightSolver,
    FollowTerrainHeightSolver(FollowTerrainHeightSolver),
    GravityHeightSolver(GravityHeightSolver),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumIconElement {
    IconElementCircleMaskeExtension,
    IconElementGradientExtension,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumMovement {
    AcceleratingMovement(AcceleratingMovement),
    FixedSpeedMovement(FixedSpeedMovement),
    FixedSpeedSplineMovement(FixedSpeedSplineMovement),
    FixedTimeMovement(FixedTimeMovement),
    FixedTimeSplineMovement(FixedTimeSplineMovement),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumTargeterDefinition {
    TargeterDefinitionAoe(TargeterDefinitionAoe),
    TargeterDefinitionLine(TargeterDefinitionLine),
    TargeterDefinitionRange(TargeterDefinitionRange),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumUiDraggable {
    UiDraggableBasic(UiDraggableBasic),
    UiDraggableElementGroupDrag(UiDraggableElementGroupDrag),
    UiDraggableProxyElementDrag(UiDraggableProxyElementDrag),
    UiDraggableSceneDrag(UiDraggableSceneDrag),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumUiPosition {
    UiPositionFullScreen,
    UiPositionPolygon(UiPositionPolygon),
    UiPositionRect(UiPositionRect),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumVfxPrimitive {
    VfxPrimitiveArbitraryQuad,
    VfxPrimitiveAttachedMesh(VfxPrimitiveAttachedMesh),
    VfxPrimitiveCameraUnitQuad,
    VfxPrimitiveMesh(VfxPrimitiveMesh),
    VfxPrimitiveRay,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumVfxShape {
    Unk0xee39916f(Unk0xee39916f),
    VfxShapeBox(VfxShapeBox),
    VfxShapeCylinder(VfxShapeCylinder),
    VfxShapeLegacy(VfxShapeLegacy),
    VfxShapeSphere(VfxShapeSphere),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExponentSubPartsCalculationPart {
    pub part1: NamedDataValueCalculationPart,
    pub part2: NumberCalculationPart,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedSpeedMovement {
    pub m_offset_initial_target_height: Option<f32>,
    pub m_project_target_to_cast_range: Option<bool>,
    pub m_speed: f32,
    pub m_start_bone_name: Option<String>,
    pub m_target_bone_name: Option<String>,
    pub m_target_height_augment: Option<f32>,
    pub m_tracks_target: Option<bool>,
    pub m_use_ground_height_at_target: Option<bool>,
    pub m_use_height_offset_at_end: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedSpeedSplineMovement {
    pub m_offset_initial_target_height: Option<f32>,
    pub m_speed: f32,
    pub m_spline_info: HermiteSplineInfo,
    pub m_start_bone_name: Option<String>,
    pub m_target_bone_name: Option<String>,
    pub m_target_height_augment: Option<f32>,
    pub m_use_height_offset_at_end: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedTimeMovement {
    pub m_offset_initial_target_height: Option<f32>,
    pub m_start_bone_name: String,
    pub m_target_height_augment: Option<f32>,
    pub m_travel_time: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedTimeSplineMovement {
    pub m_spline_info: HermiteSplineInfo,
    pub m_start_bone_name: String,
    pub m_target_bone_name: String,
    pub m_tracks_target: bool,
    pub m_travel_time: f32,
    pub m_use_missile_position_as_origin: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlexValueVector2 {
    pub m_value: Option<ValueVector2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlexValueVector3 {
    pub m_value: ValueVector3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FloatPerSpellLevel {
    pub m_per_level_values: Vec<f32>,
    pub m_value_type: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FollowTerrainHeightSolver {
    pub m_height_offset: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameCalculation {
    pub m_display_as_percent: Option<bool>,
    pub m_expanded_tooltip_calculation_display: Option<u8>,
    pub m_formula_parts: Option<Vec<EnumAbilityResourceByCoefficientCalculationPart>>,
    pub m_multiplier: Option<EnumAbilityResourceByCoefficientCalculationPart>,
    pub m_precision: Option<i32>,
    pub m_simple_tooltip_calculation_display: Option<u8>,
    pub result_modifier: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GravityHeightSolver {
    pub m_gravity: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HermiteSplineInfo {
    pub m_control_point1: Vec3,
    pub m_control_point2: Vec3,
    pub m_start_position_offset: Option<Vec3>,
    pub m_use_missile_position_as_origin: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueFloat {
    pub constant_value: Option<f32>,
    pub dynamics: VfxAnimatedFloatVariableData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueVector2 {
    pub constant_value: Option<Vec2>,
    pub dynamics: VfxAnimatedVector2fVariableData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueVector3 {
    pub constant_value: Option<Vec3>,
    pub dynamics: VfxAnimatedVector3fVariableData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LooseUiTextureData {
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LooseUiTextureData3SliceH {
    pub edge_sizes_left_right: Option<Vec2>,
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LooseUiTextureData3SliceV {
    pub edge_sizes_top_bottom: Vec2,
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LooseUiTextureData9Slice {
    pub edge_sizes_left_right: Option<Vec2>,
    pub edge_sizes_top_bottom: Option<Vec2>,
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MissileSpecification {
    pub behaviors: Option<Vec<EnumCastOnHit>>,
    pub height_solver: Option<EnumHeightSolver>,
    pub m_missile_width: Option<f32>,
    pub movement_component: EnumMovement,
    pub vertical_facing: Option<EnumFacing>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NamedDataValueCalculationPart {
    pub m_data_value: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberCalculationPart {
    pub m_number: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ObjectTags {
    pub m_object_tag_list: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductOfSubPartsCalculationPart {
    pub m_part1: Box<EnumAbilityResourceByCoefficientCalculationPart>,
    pub m_part2: Box<EnumAbilityResourceByCoefficientCalculationPart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataResource {
    pub always_snap_facing: Option<bool>,
    pub b_have_hit_bone: Option<bool>,
    pub b_have_hit_effect: Option<bool>,
    pub b_is_toggle_spell: Option<bool>,
    pub can_cast_or_queue_while_casting: Option<bool>,
    pub can_cast_while_disabled: Option<bool>,
    pub cannot_be_suppressed: Option<bool>,
    pub cant_cast_while_rooted: Option<bool>,
    pub cast_cone_angle: Option<f32>,
    pub cast_cone_distance: Option<f32>,
    pub cast_frame: Option<f32>,
    pub cast_radius: Option<Vec<f32>>,
    pub cast_radius_secondary: Option<Vec<f32>>,
    pub cast_range: Option<Vec<f32>>,
    pub cast_range_display_override: Option<Vec<f32>>,
    pub cast_range_use_bounding_boxes: Option<bool>,
    pub cooldown_time: Option<Vec<f32>>,
    pub data_values: Option<Vec<SpellDataValue>>,
    pub delay_cast_offset_percent: Option<f32>,
    pub delay_total_time_percent: Option<f32>,
    pub flags: Option<u32>,
    pub lua_on_missile_update_distance_interval: Option<f32>,
    pub m_affects_status_flags: Option<u32>,
    pub m_affects_type_flags: Option<u32>,
    pub m_alternate_name: Option<String>,
    pub m_ammo_used: Option<Vec<i32>>,
    pub m_animation_loop_name: Option<String>,
    pub m_animation_name: Option<String>,
    pub m_apply_attack_damage: Option<bool>,
    pub m_apply_material_on_hit_sound: Option<bool>,
    pub m_can_move_while_channeling: Option<bool>,
    pub m_cant_cancel_while_channeling: Option<bool>,
    pub m_cant_cancel_while_winding_up: Option<bool>,
    pub m_cant_cancel_while_winding_up_targeting_champion: Option<bool>,
    pub m_cast_range_growth_duration: Option<Vec<f32>>,
    pub m_cast_range_growth_max: Option<Vec<f32>>,
    pub m_cast_time: Option<f32>,
    pub m_cast_type: Option<u32>,
    pub m_caster_position_end_of_cast_update: Option<u8>,
    pub m_casting_breaks_stealth: Option<bool>,
    pub m_channel_duration: Option<Vec<f32>>,
    pub m_channel_is_interrupted_by_attacking: Option<bool>,
    pub m_channel_is_interrupted_by_disables: Option<bool>,
    pub m_client_data: Option<SpellDataResourceClient>,
    pub m_coefficient: Option<f32>,
    pub m_coefficient2: Option<f32>,
    pub m_dimension_behavior: Option<u8>,
    pub m_disable_cast_bar: Option<bool>,
    pub m_do_not_need_to_face_target: Option<bool>,
    pub m_does_not_consume_mana: Option<bool>,
    pub m_doesnt_break_channels: Option<bool>,
    pub m_float_vars_decimals: Option<Vec<i32>>,
    pub m_hide_range_indicator_when_casting: Option<bool>,
    pub m_hit_bone_name: Option<String>,
    pub m_hit_effect_key: Option<u32>,
    pub m_hit_effect_orient_type: Option<u32>,
    pub m_ignore_range_check: Option<bool>,
    pub m_img_icon_name: Option<Vec<String>>,
    pub m_is_disabled_while_dead: Option<bool>,
    pub m_line_width: Option<f32>,
    pub m_look_at_policy: Option<u32>,
    pub m_minimap_icon_display_flag: Option<u16>,
    pub m_minimap_icon_rotation: Option<bool>,
    pub m_missile_effect_key: Option<u32>,
    pub m_missile_spec: Option<MissileSpecification>,
    pub m_particle_start_offset: Option<Vec3>,
    pub m_pingable_while_disabled: Option<bool>,
    pub m_post_cast_lockout_delta_time: Option<f32>,
    pub m_project_target_to_cast_range: Option<bool>,
    pub m_required_unit_tags: Option<ObjectTags>,
    pub m_roll_for_critical_hit: Option<bool>,
    pub m_show_channel_bar: Option<bool>,
    pub m_spell_calculations: Option<HashMap<u32, GameCalculation>>,
    pub m_spell_cooldown_or_sealed_queue_threshold: Option<f32>,
    pub m_spell_reveals_champion: Option<bool>,
    pub m_spell_tags: Option<Vec<String>>,
    pub m_targeting_type_data: Option<EnumDirection>,
    pub m_turn_speed_scalar: Option<f32>,
    pub m_update_rotation_when_casting: Option<bool>,
    pub m_use_autoattack_cast_time_data: Option<UseAutoattackCastTimeData>,
    pub m_use_minimap_targeting: Option<bool>,
    pub mana: Option<Vec<f32>>,
    pub missile_speed: Option<f32>,
    pub selection_priority: Option<u32>,
    pub spell_event_to_audio_event_suffix: Option<HashMap<u32, String>>,
    pub unk_0xabe507b9: Option<u32>,
    pub use_animator_framerate: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataResourceClient {
    pub m_left_click_spell_action: Option<u32>,
    pub m_right_click_spell_action: Option<u32>,
    pub m_targeter_definitions: Option<Vec<EnumTargeterDefinition>>,
    pub m_tooltip_data: Option<TooltipInstanceSpell>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataValue {
    pub m_name: String,
    pub m_values: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct SpellObject {
    pub m_buff: Option<BuffData>,
    pub m_script_name: String,
    pub m_spell: SpellDataResource,
    pub object_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatByCoefficientCalculationPart {
    pub m_coefficient: f32,
    pub m_stat: u8,
    pub m_stat_formula: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatByNamedDataValueCalculationPart {
    pub m_data_value: u32,
    pub m_stat: u8,
    pub m_stat_formula: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatBySubPartCalculationPart {
    pub m_stat: u8,
    pub m_subpart: NamedDataValueCalculationPart,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatEfficiencyPerHundred {
    pub m_bonus_stat_for_efficiency: f32,
    pub m_data_value: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialDef {
    pub dynamic_material: Option<DynamicMaterialDef>,
    pub name: String,
    pub param_values: Vec<StaticMaterialShaderParamDef>,
    pub r#type: u32,
    pub sampler_values: Option<Vec<StaticMaterialShaderSamplerDef>>,
    pub switches: Option<Vec<StaticMaterialSwitchDef>>,
    pub techniques: Vec<StaticMaterialTechniqueDef>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialPassDef {
    pub blend_enable: bool,
    pub cull_enable: bool,
    pub depth_enable: bool,
    pub dst_alpha_blend_factor: u32,
    pub dst_color_blend_factor: u32,
    pub shader: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialShaderParamDef {
    pub name: String,
    pub value: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialShaderSamplerDef {
    pub address_u: Option<u32>,
    pub address_v: Option<u32>,
    pub address_w: Option<u32>,
    pub texture_name: String,
    pub texture_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialSwitchDef {
    pub name: String,
    pub on: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialTechniqueDef {
    pub name: String,
    pub passes: Vec<StaticMaterialPassDef>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubPartScaledProportionalToStat {
    pub m_ratio: f32,
    pub m_stat: Option<u8>,
    pub m_style_tag: Option<String>,
    pub m_style_tag_if_scaled: Option<String>,
    pub m_subpart: Box<EnumAbilityResourceByCoefficientCalculationPart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SumOfSubPartsCalculationPart {
    pub m_subparts: Vec<Box<EnumAbilityResourceByCoefficientCalculationPart>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionAoe {
    pub center_locator: DrawablePositionLocator,
    pub override_radius: Option<FloatPerSpellLevel>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionLine {
    pub end_locator: DrawablePositionLocator,
    pub line_width: Option<FloatPerSpellLevel>,
    pub override_base_range: Option<FloatPerSpellLevel>,
    pub texture_base_override_name: Option<String>,
    pub texture_target_override_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionRange {
    pub hide_with_line_indicator: Option<bool>,
    pub override_base_range: Option<FloatPerSpellLevel>,
    pub use_caster_bounding_box: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceList {
    pub elements: Option<Vec<TooltipInstanceListElement>>,
    pub level_count: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceListElement {
    pub multiplier: Option<f32>,
    pub name_override: String,
    pub r#type: String,
    pub style: Option<u32>,
    pub type_index: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceSpell {
    pub m_format: u32,
    pub m_lists: Option<HashMap<String, TooltipInstanceList>>,
    pub m_loc_keys: Option<HashMap<String, String>>,
    pub m_object_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiDraggableBasic {
    pub use_sticky_drag: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiDraggableElementGroupDrag {
    pub use_sticky_drag: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiDraggableProxyElementDrag {
    pub proxy_element: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiDraggableSceneDrag {
    pub use_sticky_drag: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectAnimationData {
    pub enabled: Option<bool>,
    pub frames_per_second: Option<f32>,
    pub layer: Option<u32>,
    pub m_finish_behavior: Option<u8>,
    pub m_per_pixel_uvs_x: Option<bool>,
    pub name: String,
    pub number_of_frames_per_row_in_atlas: Option<f32>,
    pub position: UiPositionRect,
    pub scene: u32,
    pub texture_data: EnumData,
    pub total_number_of_frames: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupButtonData {
    pub active_tooltip_tra_key: Option<String>,
    pub add_text_size_to_hit_region: Option<bool>,
    pub click_release_particle_element: Option<u32>,
    pub clicked_state_elements: Option<UiElementGroupButtonState>,
    pub default_state_elements: Option<UiElementGroupButtonState>,
    pub elements: Vec<u32>,
    pub hit_region_element: u32,
    pub hover_state_elements: Option<UiElementGroupButtonState>,
    pub inactive_selected_state_elements: Option<UiElementGroupButtonState>,
    pub inactive_state_elements: Option<UiElementGroupButtonState>,
    pub inactive_tooltip_tra_key: Option<String>,
    pub is_active: Option<bool>,
    pub is_enabled: Option<bool>,
    pub is_focusable: Option<bool>,
    pub is_selected: Option<bool>,
    pub name: String,
    pub scene: u32,
    pub selected_clicked_state_elements: Option<UiElementGroupButtonState>,
    pub selected_hover_state_elements: Option<UiElementGroupButtonState>,
    pub selected_state_elements: Option<UiElementGroupButtonState>,
    pub selected_tooltip_tra_key: Option<String>,
    pub sound_events: Option<UiElementGroupButtonSoundEvents>,
    pub tab_order: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupButtonSoundEvents {
    pub mouse_down_event: Option<String>,
    pub mouse_down_on_inactive: Option<String>,
    pub mouse_down_selected: Option<String>,
    pub mouse_up_event: Option<String>,
    pub mouse_up_selected: Option<String>,
    pub roll_over_event: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupButtonState {
    pub display_element_list: Option<Vec<u32>>,
    pub text_element: Option<u32>,
    pub text_frame_element: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct UiElementIconData {
    pub block_input_events: Option<bool>,
    pub color: Option<[u8; 4]>,
    pub drag_type: Option<EnumUiDraggable>,
    pub enabled: Option<bool>,
    pub extension: Option<EnumIconElement>,
    pub fill_type: Option<u32>,
    pub flip_x: Option<bool>,
    pub flip_y: Option<bool>,
    pub layer: Option<u32>,
    pub material: Option<u32>,
    pub name: String,
    pub per_pixel_uvs_x: Option<bool>,
    pub position: EnumUiPosition,
    pub scene: u32,
    pub texture_data: Option<EnumData>,
    pub use_alpha: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementRect {
    pub position: Option<Vec2>,
    pub size: Option<Vec2>,
    pub source_resolution_height: Option<u16>,
    pub source_resolution_width: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct UiElementRegionData {
    pub block_input_events: Option<bool>,
    pub drag_type: Option<EnumUiDraggable>,
    pub enabled: Option<bool>,
    pub layer: Option<u32>,
    pub name: String,
    pub position: Option<EnumUiPosition>,
    pub scene: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiPositionPolygon {
    pub anchors: AnchorSingle,
    pub polygon_vertices: Vec<Vec2>,
    pub ui_rect: UiElementRect,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiPositionRect {
    pub anchors: Option<EnumAnchor>,
    pub disable_pixel_snapping_x: Option<bool>,
    pub disable_pixel_snapping_y: Option<bool>,
    pub disable_resolution_downscale: Option<bool>,
    pub ignore_global_scale: Option<bool>,
    pub ignore_safe_zone: Option<bool>,
    pub ui_rect: Option<UiElementRect>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x382277da {
    pub m_subparts: Vec<Box<EnumAbilityResourceByCoefficientCalculationPart>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x8a96ea3c {
    pub m_subparts: Vec<Box<EnumAbilityResourceByCoefficientCalculationPart>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xee39916f {
    pub emit_offset: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf090d2e7 {
    pub unk_0x9567c2a: Option<u8>,
    pub unk_0xa567dbd: Option<u8>,
    pub unk_0xbe0de52: Option<u8>,
    pub unk_0xce0dfe5: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf3cbe7b2 {
    pub m_spell_calculation_key: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UseAutoattackCastTimeData {
    pub m_autoattack_cast_time_calculation: GameCalculation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValueColor {
    pub constant_value: Option<Vec4>,
    pub dynamics: Option<VfxAnimatedColorVariableData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValueFloat {
    pub constant_value: Option<f32>,
    pub dynamics: Option<VfxAnimatedFloatVariableData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValueVector2 {
    pub constant_value: Option<Vec2>,
    pub dynamics: Option<VfxAnimatedVector2fVariableData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValueVector3 {
    pub constant_value: Option<Vec3>,
    pub dynamics: Option<VfxAnimatedVector3fVariableData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAlphaErosionDefinitionData {
    pub erosion_drive_curve: Option<ValueFloat>,
    pub erosion_feather_in: Option<f32>,
    pub erosion_feather_out: Option<f32>,
    pub erosion_map_address_mode: Option<u8>,
    pub erosion_map_channel_mixer: Option<ValueColor>,
    pub erosion_map_name: Option<String>,
    pub erosion_slice_width: Option<f32>,
    pub linger_erosion_drive_curve: Option<ValueFloat>,
    pub use_linger_erosion_drive_curve: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedColorVariableData {
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub times: Vec<f32>,
    pub values: Vec<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedFloatVariableData {
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub times: Vec<f32>,
    pub values: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedVector2fVariableData {
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub times: Vec<f32>,
    pub values: Vec<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedVector3fVariableData {
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub times: Vec<f32>,
    pub values: Vec<Vec3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAssetRemap {
    pub new_asset: Option<String>,
    pub old_asset: Option<u32>,
    pub r#type: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxChildIdentifier {
    pub effect: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxChildParticleSetDefinitionData {
    pub children_identifiers: Vec<VfxChildIdentifier>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxDistortionDefinitionData {
    pub distortion: f32,
    pub distortion_mode: Option<u8>,
    pub normal_map_texture: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterDefinitionData {
    pub acceleration: Option<ValueVector3>,
    pub alpha_erosion_definition: Option<VfxAlphaErosionDefinitionData>,
    pub alpha_ref: Option<u8>,
    pub bind_weight: Option<ValueFloat>,
    pub birth_acceleration: Option<ValueVector3>,
    pub birth_color: Option<ValueColor>,
    pub birth_drag: Option<ValueVector3>,
    pub birth_frame_rate: Option<ValueFloat>,
    pub birth_orbital_velocity: Option<ValueVector3>,
    pub birth_rotation0: Option<ValueVector3>,
    pub birth_rotational_acceleration: Option<ValueVector3>,
    pub birth_rotational_velocity0: Option<ValueVector3>,
    pub birth_scale0: Option<ValueVector3>,
    pub birth_uv_offset: Option<ValueVector2>,
    pub birth_uv_rotate_rate: Option<ValueFloat>,
    pub birth_uv_scroll_rate: Option<ValueVector2>,
    pub birth_velocity: Option<ValueVector3>,
    pub blend_mode: Option<u8>,
    pub censor_modulate_value: Option<Vec4>,
    pub chance_to_not_exist: Option<f32>,
    pub child_particle_set_definition: Option<VfxChildParticleSetDefinitionData>,
    pub color: Option<ValueColor>,
    pub color_look_up_offsets: Option<Vec2>,
    pub color_look_up_scales: Option<Vec2>,
    pub color_look_up_type_x: Option<u8>,
    pub color_look_up_type_y: Option<u8>,
    pub color_render_flags: Option<u8>,
    pub colorblind_visibility: Option<u8>,
    pub depth_bias_factors: Option<Vec2>,
    pub direction_velocity_min_scale: Option<f32>,
    pub direction_velocity_scale: Option<f32>,
    pub disable_backface_cull: Option<bool>,
    pub disabled: Option<bool>,
    pub distortion_definition: Option<VfxDistortionDefinitionData>,
    pub drag: Option<ValueVector3>,
    pub emission_mesh_scale: Option<f32>,
    pub emitter_linger: Option<f32>,
    pub emitter_name: String,
    pub emitter_position: Option<ValueVector3>,
    pub falloff_texture: Option<String>,
    pub field_collection_definition: Option<VfxFieldCollectionDefinitionData>,
    pub filtering: Option<VfxEmitterFiltering>,
    pub flex_birth_rotational_velocity0: Option<FlexValueVector3>,
    pub flex_birth_uv_offset: Option<FlexValueVector2>,
    pub flex_shape_definition: Option<VfxFlexShapeDefinitionData>,
    pub frame_rate: Option<f32>,
    pub has_post_rotate_orientation: Option<bool>,
    pub has_variable_start_time: Option<bool>,
    pub importance: Option<u8>,
    pub is_direction_oriented: Option<bool>,
    pub is_emitter_space: Option<bool>,
    pub is_ground_layer: Option<bool>,
    pub is_local_orientation: Option<bool>,
    pub is_random_start_frame: Option<bool>,
    pub is_rotation_enabled: Option<bool>,
    pub is_single_particle: Option<bool>,
    pub is_uniform_scale: Option<bool>,
    pub lifetime: Option<f32>,
    pub linger: Option<VfxLingerDefinitionData>,
    pub maximum_rate_by_velocity: Option<f32>,
    pub mesh_render_flags: Option<u8>,
    pub misc_render_flags: Option<u8>,
    pub modulation_factor: Option<Vec4>,
    pub num_frames: Option<u16>,
    pub palette_definition: Option<VfxPaletteDefinitionData>,
    pub particle_color_texture: Option<String>,
    pub particle_is_local_orientation: Option<bool>,
    pub particle_lifetime: Option<ValueFloat>,
    pub particle_linger: Option<f32>,
    pub particle_linger_type: Option<u8>,
    pub particle_uv_rotate_rate: Option<IntegratedValueFloat>,
    pub particle_uv_scroll_rate: Option<IntegratedValueVector2>,
    pub pass: Option<i16>,
    pub period: Option<f32>,
    pub post_rotate_orientation_axis: Option<Vec3>,
    pub primitive: Option<EnumVfxPrimitive>,
    pub rate: Option<ValueFloat>,
    pub rate_by_velocity_function: Option<ValueVector2>,
    pub reflection_definition: Option<VfxReflectionDefinitionData>,
    pub rotation0: Option<IntegratedValueVector3>,
    pub rotation_override: Option<Vec3>,
    pub scale0: Option<ValueVector3>,
    pub scale_override: Option<Vec3>,
    pub soft_particle_params: Option<VfxSoftParticleDefinitionData>,
    pub sort_emitters_by_pos: Option<bool>,
    pub spawn_shape: Option<EnumVfxShape>,
    pub start_frame: Option<u16>,
    pub stencil_mode: Option<u8>,
    pub stencil_ref: Option<u8>,
    pub tex_address_mode_base: Option<u8>,
    pub tex_div: Option<Vec2>,
    pub texture: Option<String>,
    pub texture_flip_u: Option<bool>,
    pub texture_flip_v: Option<bool>,
    pub texture_mult: Option<VfxTextureMultDefinitionData>,
    pub time_active_during_period: Option<f32>,
    pub time_before_first_emission: Option<f32>,
    pub translation_override: Option<Vec3>,
    pub unk_0xcb13aff1: Option<f32>,
    pub unk_0xd1ee8634: Option<bool>,
    pub use_emission_mesh_normal_for_birth: Option<bool>,
    pub use_navmesh_mask: Option<bool>,
    pub uv_mode: Option<u8>,
    pub uv_rotation: Option<ValueFloat>,
    pub uv_scale: Option<ValueVector2>,
    pub uv_scroll_clamp: Option<bool>,
    pub uv_transform_center: Option<Vec2>,
    pub velocity: Option<ValueVector3>,
    pub world_acceleration: Option<IntegratedValueVector3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterFiltering {
    pub keywords_excluded: Option<Vec<String>>,
    pub spectator_policy: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldAccelerationDefinitionData {
    pub acceleration: ValueVector3,
    pub is_local_space: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldAttractionDefinitionData {
    pub acceleration: ValueFloat,
    pub position: Option<ValueVector3>,
    pub radius: ValueFloat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldCollectionDefinitionData {
    pub field_acceleration_definitions: Option<Vec<VfxFieldAccelerationDefinitionData>>,
    pub field_attraction_definitions: Option<Vec<VfxFieldAttractionDefinitionData>>,
    pub field_drag_definitions: Option<Vec<VfxFieldDragDefinitionData>>,
    pub field_noise_definitions: Option<Vec<VfxFieldNoiseDefinitionData>>,
    pub field_orbital_definitions: Option<Vec<VfxFieldOrbitalDefinitionData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldDragDefinitionData {
    pub radius: ValueFloat,
    pub strength: ValueFloat,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldNoiseDefinitionData {
    pub axis_fraction: Option<Vec3>,
    pub frequency: Option<ValueFloat>,
    pub position: Option<ValueVector3>,
    pub radius: Option<ValueFloat>,
    pub velocity_delta: Option<ValueFloat>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldOrbitalDefinitionData {
    pub direction: ValueVector3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFlexShapeDefinitionData {
    pub scale_birth_scale_by_bound_object_size: Option<f32>,
    pub scale_emit_offset_by_bound_object_size: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxLingerDefinitionData {
    pub use_keyed_linger_acceleration: Option<bool>,
    pub use_keyed_linger_velocity: Option<bool>,
    pub use_linger_rotation: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxMeshDefinitionData {
    pub m_animation_name: Option<String>,
    pub m_mesh_name: Option<String>,
    pub m_mesh_skeleton_name: Option<String>,
    pub m_simple_mesh_name: Option<String>,
    pub m_submeshes_to_draw_always: Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPaletteDefinitionData {
    pub palette_count: Option<i32>,
    pub palette_selector: Option<ValueVector3>,
    pub palette_texture: Option<String>,
    pub palette_texture_address_mode: Option<u8>,
    pub palette_u_animation_curve: Option<ValueFloat>,
    pub palette_v_animation_curve: Option<ValueFloat>,
    pub pallete_src_mix_color: Option<ValueColor>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveAttachedMesh {
    pub m_mesh: VfxMeshDefinitionData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveMesh {
    pub m_mesh: VfxMeshDefinitionData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxProbabilityTableData {
    pub key_times: Option<Vec<f32>>,
    pub key_values: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxReflectionDefinitionData {
    pub fresnel: Option<f32>,
    pub fresnel_color: Option<Vec4>,
    pub reflection_fresnel: Option<f32>,
    pub reflection_fresnel_color: Option<Vec4>,
    pub reflection_map_texture: Option<String>,
    pub reflection_opacity_direct: Option<f32>,
    pub reflection_opacity_glancing: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeBox {
    pub flags: Option<u8>,
    pub size: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeCylinder {
    pub flags: Option<u8>,
    pub height: Option<f32>,
    pub radius: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeLegacy {
    pub emit_offset: Option<ValueVector3>,
    pub emit_rotation_angles: Option<Vec<ValueFloat>>,
    pub emit_rotation_axes: Option<Vec<Vec3>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeSphere {
    pub flags: Option<u8>,
    pub radius: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxSoftParticleDefinitionData {
    pub begin_in: Option<f32>,
    pub delta_in: Option<f32>,
    pub delta_out: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct VfxSystemDefinitionData {
    pub asset_remapping_table: Option<Vec<VfxAssetRemap>>,
    pub clock_to_use: Option<u8>,
    pub complex_emitter_definition_data: Option<Vec<VfxEmitterDefinitionData>>,
    pub drawing_layer: Option<u8>,
    pub flags: Option<u16>,
    pub hud_anchor_position_from_world_projection: Option<bool>,
    pub hud_layer_dimension: Option<f32>,
    pub m_eye_candy: Option<bool>,
    pub override_scale_cap: Option<f32>,
    pub particle_name: String,
    pub particle_path: String,
    pub scale_dynamically_with_attached_bone: Option<bool>,
    pub sound_on_create_default: Option<String>,
    pub sound_persistent_default: Option<String>,
    pub transform: Option<Mat4>,
    pub unk_0x9836cd87: u8,
    pub visibility_radius: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxTextureMultDefinitionData {
    pub birth_uv_offset_mult: Option<ValueVector2>,
    pub birth_uv_rotate_rate_mult: Option<ValueFloat>,
    pub birth_uv_scroll_rate_mult: Option<ValueVector2>,
    pub emitter_uv_scroll_rate_mult: Option<Vec2>,
    pub flex_birth_uv_scroll_rate_mult: Option<FlexValueVector2>,
    pub particle_integrated_uv_rotate_mult: Option<IntegratedValueFloat>,
    pub particle_integrated_uv_scroll_mult: Option<IntegratedValueVector2>,
    pub tex_address_mode_mult: Option<u8>,
    pub tex_div_mult: Option<Vec2>,
    pub texture_mult: Option<String>,
    pub texture_mult_filp_u: Option<bool>,
    pub texture_mult_filp_v: Option<bool>,
    pub uv_rotation_mult: Option<ValueFloat>,
    pub uv_scale_mult: Option<ValueVector2>,
    pub uv_scroll_alpha_mult: Option<bool>,
    pub uv_scroll_clamp_mult: Option<bool>,
    pub uv_transform_center_mult: Option<Vec2>,
}

