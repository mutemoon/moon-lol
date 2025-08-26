use std::collections::HashMap;

use bevy::math::{Mat4, Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum SwitchMaterialDriverElementMCondition {
    IsDeadDynamicMaterialBoolDriver,
    IsAnimationPlayingDynamicMaterialBoolDriver(IsAnimationPlayingDynamicMaterialBoolDriver),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SwitchMaterialDriverElementMValue {
    FloatLiteralMaterialDriver(FloatLiteralMaterialDriver),
    FloatGraphMaterialDriver(FloatGraphMaterialDriver),
    LerpMaterialDriver(LerpMaterialDriver),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ParametricClipDataMEventDataMap {
    SoundEventData(SoundEventData),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ParametricClipDataUpdater {
    LookAtInterestAngleParametricUpdater,
    TurnAngleParametricUpdater,
    LookAtSpellTargetAngleParametricUpdater,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ColorChooserMaterialDriverMBoolDriver {
    IsDeadDynamicMaterialBoolDriver,
    IsAnimationPlayingDynamicMaterialBoolDriver(IsAnimationPlayingDynamicMaterialBoolDriver),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BarracksMinionConfigWaveBehavior {
    ConstantWaveBehavior(ConstantWaveBehavior),
    InhibitorWaveBehavior(InhibitorWaveBehavior),
    TimedVariableWaveBehavior(TimedVariableWaveBehavior),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MapContainerComponents {
    Unk0xcf4a55da(Unk0xcf4a55da),
    Unk0xcdb1c8f6(Unk0xcdb1c8f6),
    MapSunProperties(MapSunProperties),
    Unk0x0,
    MapTerrainPaint(MapTerrainPaint),
    MapBakeProperties(MapBakeProperties),
    MapNavGrid(MapNavGrid),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SequencerClipDataMEventDataMap {
    SubmeshVisibilityEventData(SubmeshVisibilityEventData),
    ParticleEventData(ParticleEventData),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TimedWaveBehaviorInfoBehavior {
    RotatingWaveBehavior(RotatingWaveBehavior),
    ConstantWaveBehavior(ConstantWaveBehavior),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AnimationGraphDataMSyncGroupDataMap {
    SyncGroupData,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AnimationGraphDataMTrackDataMap {
    TrackData(TrackData),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AnimationGraphDataMBlendDataTable {
    TimeBlendData(TimeBlendData),
    TransitionClipBlendData(TransitionClipBlendData),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AnimationGraphDataMMaskDataMap {
    MaskData(MaskData),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AnimationGraphDataMClipDataMap {
    SequencerClipData(SequencerClipData),
    ParametricClipData(ParametricClipData),
    SelectorClipData(SelectorClipData),
    AtomicClipData(AtomicClipData),
    ParallelClipData(ParallelClipData),
    ConditionFloatClipData(ConditionFloatClipData),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LerpMaterialDriverMBoolDriver {
    IsAnimationPlayingDynamicMaterialBoolDriver(IsAnimationPlayingDynamicMaterialBoolDriver),
    IsDeadDynamicMaterialBoolDriver,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MissileSpecificationMovementComponent {
    AcceleratingMovement(AcceleratingMovement),
    FixedTimeMovement(FixedTimeMovement),
    FixedSpeedMovement(FixedSpeedMovement),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MissileSpecificationHeightSolver {
    GravityHeightSolver(GravityHeightSolver),
    BlendedLinearHeightSolver,
    FollowTerrainHeightSolver(FollowTerrainHeightSolver),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MissileSpecificationVerticalFacing {
    VerticalFacingFaceTarget,
    VeritcalFacingMatchVelocity,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MissileSpecificationBehaviors {
    CastOnMovementComplete,
    CastOnHit,
    DestroyOnMovementComplete,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MissileSpecificationVisibilityComponent {
    EnterFowVisibility(EnterFowVisibility),
    Defaultvisibility(Defaultvisibility),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ContextualConditionCharacterMChildConditions {
    ContextualConditionCharacterName(ContextualConditionCharacterName),
    Unk0xb6da23cb,
    ContextualConditionCharacterMetadata(ContextualConditionCharacterMetadata),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SpellDataResourceMTargetingTypeData {
    SelfAoe,
    Direction,
    LocationClamped,
    Area,
    Location,
    TargetOrLocation,
    MySelf,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SpellDataResourceDataValuesModeOverride {
    SpellDataValueVector(SpellDataValueVector),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SpellDataResourceMSpellCalculations {
    GameCalculationModified(GameCalculationModified),
    GameCalculation(GameCalculation),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameCalculationMFormulaParts {
    StatByCoefficientCalculationPart(StatByCoefficientCalculationPart),
    NamedDataValueCalculationPart(NamedDataValueCalculationPart),
    Unk0xf3cbe7b2(Unk0xf3cbe7b2),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CharacterToolDataMapAiPresence {
    ToolAiPresence,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MapPlaceableContainerItems {
    Unk0x111a9fcc(Unk0x111a9fcc),
    MapAudio(MapAudio),
    Unk0x7ad3dda(Unk0x7ad3dda),
    MapCubemapProbe(MapCubemapProbe),
    MapParticle(MapParticle),
    Unk0x3c995caf(Unk0x3c995caf),
    MapGroup(MapGroup),
    GdsMapObject(GdsMapObject),
    MapLocator(MapLocator),
    MapScriptLocator(MapScriptLocator),
    Unk0xff6e8118(Unk0xff6e8118),
    Unk0x3c2bf0c0(Unk0x3c2bf0c0),
    Unk0x42239bf8(Unk0x42239bf8),
    Unk0xc71ee7fb(Unk0xc71ee7fb),
    Unk0x0,
    Unk0xf4a21c35(Unk0xf4a21c35),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ContextualActionDataMSituations {
    ContextualSituation(ContextualSituation),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ConditionFloatClipDataUpdater {
    LookAtInterestAngleParametricUpdater,
    MoveSpeedParametricUpdater,
    LookAtInterestDistanceParametricUpdater,
    LookAtSpellTargetAngleParametricUpdater,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AtomicClipDataMEventDataMap {
    SubmeshVisibilityEventData(SubmeshVisibilityEventData),
    SoundEventData(SoundEventData),
    EnableLookAtEventData(EnableLookAtEventData),
    FadeEventData(FadeEventData),
    ParticleEventData(ParticleEventData),
    ConformToPathEventData(ConformToPathEventData),
    JointSnapEventData(JointSnapEventData),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DynamicMaterialParameterDefDriver {
    SwitchMaterialDriver(SwitchMaterialDriver),
    BlendingSwitchMaterialDriver(BlendingSwitchMaterialDriver),
    ColorChooserMaterialDriver(ColorChooserMaterialDriver),
    SineMaterialDriver(SineMaterialDriver),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SpellDataResourceClientMTargeterDefinitions {
    TargeterDefinitionRange(TargeterDefinitionRange),
    TargeterDefinitionLine(TargeterDefinitionLine),
    TargeterDefinitionMinimap(TargeterDefinitionMinimap),
    TargeterDefinitionAoe(TargeterDefinitionAoe),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TooltipInstanceSpellMLists {
    TooltipInstanceList(TooltipInstanceList),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum VfxEmitterDefinitionDataSpawnShape {
    VfxShapeSphere(VfxShapeSphere),
    VfxShapeBox(VfxShapeBox),
    Unk0xee39916f(Unk0xee39916f),
    VfxShapeLegacy(VfxShapeLegacy),
    VfxShapeCylinder(VfxShapeCylinder),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum VfxEmitterDefinitionDataPrimitive {
    VfxPrimitiveCameraTrail(VfxPrimitiveCameraTrail),
    VfxPrimitiveArbitraryTrail(VfxPrimitiveArbitraryTrail),
    VfxPrimitiveBeam(VfxPrimitiveBeam),
    VfxPrimitivePlanarProjection(VfxPrimitivePlanarProjection),
    Unk0x8df5fcf7,
    VfxPrimitiveArbitraryQuad,
    VfxPrimitiveAttachedMesh(VfxPrimitiveAttachedMesh),
    VfxPrimitiveMesh(VfxPrimitiveMesh),
    VfxPrimitiveRay,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ContextualRuleMConditions {
    ContextualConditionRuleCooldown(ContextualConditionRuleCooldown),
    ContextualConditionChanceToPlay(ContextualConditionChanceToPlay),
    ContextualConditionCharacter(ContextualConditionCharacter),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValueColor {
    pub constant_value: Option<Vec4>,
    pub dynamics: Option<VfxAnimatedColorVariableData>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnterFowVisibility {
    pub m_perception_bubble_radius: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CharacterRecord {
    pub unit_tags_string: String,
    pub wake_up_range: Option<f32>,
    pub useable_data: Option<UseableData>,
    pub spell_names: Option<Vec<String>>,
    pub crit_damage_multiplier: Option<f32>,
    pub base_damage: Option<f32>,
    pub pathfinding_collision_radius: f32,
    pub local_gold_given_on_death: Option<f32>,
    pub local_exp_given_on_death: Option<f32>,
    pub base_armor: Option<f32>,
    pub attack_speed_per_level: Option<f32>,
    pub selection_radius: Option<f32>,
    pub target_laser_effects: Option<TargetLaserComponentEffects>,
    pub global_exp_given_on_death: Option<f32>,
    pub secondary_ability_resource: Option<AbilityResourceSlotInfo>,
    pub death_event_listening_radius: Option<f32>,
    pub use_riot_relationships: Option<bool>,
    pub hit_fx_scale: Option<f32>,
    pub untargetable_spawn_time: Option<f32>,
    pub primary_ability_resource: AbilityResourceSlotInfo,
    pub health_bar_full_parallax: Option<bool>,
    pub enemy_tooltip: Option<String>,
    pub perception_bubble_radius: Option<f32>,
    pub health_bar_height: Option<f32>,
    pub passive_lua_name: Option<String>,
    pub extra_spells: Option<Vec<String>>,
    pub disguise_minimap_icon_override: Option<String>,
    pub significance: Option<f32>,
    pub global_gold_given_on_death: Option<f32>,
    pub attack_range: Option<f32>,
    pub override_gameplay_collision_radius: Option<f32>,
    pub base_hp: f32,
    pub local_gold_split_with_last_hitter: Option<bool>,
    pub tower_targeting_priority_boost: Option<f32>,
    pub friendly_tooltip: Option<String>,
    pub m_character_passive_spell: Option<u32>,
    pub m_character_name: String,
    pub attack_speed: Option<f32>,
    pub spells: Option<Vec<u32>>,
    pub on_kill_event_for_spectator: Option<u32>,
    pub unk_0xc5c48b41: Option<u8>,
    pub extra_attacks: Option<Vec<AttackSlotData>>,
    pub flags: Option<u32>,
    pub experience_radius: Option<f32>,
    pub minion_score_value: Option<f32>,
    pub m_abilities: Option<Vec<u32>>,
    pub gold_radius: Option<f32>,
    pub character_tool_data: CharacterToolData,
    pub gold_given_on_death: Option<f32>,
    pub damage_per_level: Option<f32>,
    pub selection_height: Option<f32>,
    pub m_fallback_character_name: Option<String>,
    pub attack_speed_ratio: Option<f32>,
    pub first_acquisition_range: Option<f32>,
    pub disabled_target_laser_effects: Option<TargetLaserComponentEffects>,
    pub joint_for_anim_adjusted_selection: Option<String>,
    pub base_factor_hp_regen: Option<f32>,
    pub name: String,
    pub minimap_icon_override: Option<String>,
    pub base_move_speed: Option<f32>,
    pub passive_spell: Option<String>,
    pub basic_attack: Option<AttackSlotData>,
    pub occluded_unit_selectable_distance: Option<f32>,
    pub crit_attacks: Option<Vec<AttackSlotData>>,
    pub on_kill_event_steal: Option<u32>,
    pub acquisition_range: Option<f32>,
    pub outline_b_box_expansion: Option<f32>,
    pub hover_indicator_radius: Option<f32>,
    pub base_spell_block: Option<f32>,
    pub base_static_hp_regen: f32,
    pub on_kill_event: Option<u32>,
    pub exp_given_on_death: Option<f32>,
    pub m_client_side_item_inventory: Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialTechniqueDef {
    pub name: String,
    pub passes: Vec<StaticMaterialPassDef>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FlexTypeFloat {
    pub m_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xc71ee7fb {
    pub transform: Mat4,
    pub name: u32,
    pub definition: Unk0xfde6a2d7,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SelectorPairData {
    pub m_clip_name: u32,
    pub m_probability: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GdsMapObjectBannerInfo {
    pub banner_data: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BotsSpellData {
    pub damage_tag: u32,
    pub unk_0x6d548702: GameCalculation,
    pub unk_0xec17e271: Option<Vec<Unk0xb09016f6>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwitchMaterialDriverElement {
    pub m_condition: SwitchMaterialDriverElementMCondition,
    pub m_value: SwitchMaterialDriverElementMValue,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapNavGrid {
    pub nav_grid_config: u32,
    pub nav_grid_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GearData {
    pub m_character_submeshes_to_show: Vec<u32>,
    pub enable_override_idle_effects: Option<bool>,
    pub override_idle_effects: Option<Vec<SkinCharacterDataPropertiesCharacterIdleEffect>>,
    pub m_character_submeshes_to_hide: Vec<u32>,
    pub m_vfx_resource_resolver: ResourceResolver,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxSoftParticleDefinitionData {
    pub delta_out: Option<f32>,
    pub delta_in: Option<f32>,
    pub begin_out: Option<f32>,
    pub begin_in: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParametricClipData {
    pub m_flags: Option<u32>,
    pub m_track_data_name: u32,
    pub m_sync_group_data_name: Option<u32>,
    pub m_mask_data_name: Option<u32>,
    pub m_event_data_map: Option<HashMap<u32, ParametricClipDataMEventDataMap>>,
    pub updater: ParametricClipDataUpdater,
    pub m_parametric_pair_data_list: Vec<ParametricPairData>,
    pub unk_0x69de8fca: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf4a21c35 {
    pub transform: Mat4,
    pub definition: Unk0xfcb92181,
    pub name: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapNavGridOverlays {
    pub overlays: Vec<MapNavGridOverlay>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxPaletteDefinitionData {
    pub palette_count: Option<i32>,
    pub palette_selector: Option<ValueVector3>,
    pub palette_u_animation_curve: Option<ValueFloat>,
    pub palette_texture: Option<String>,
    pub palette_texture_address_mode: Option<u8>,
    pub pallete_src_mix_color: Option<ValueColor>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimedVariableWaveBehavior {
    pub behaviors: Vec<TimedWaveBehaviorInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionRuleCooldown {
    pub m_rule_cooldown: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SkinCharacterDataPropertiesCharacterIdleEffect {
    pub bone_name: String,
    pub effect_key: Option<u32>,
    pub target_bone_name: Option<String>,
    pub effect_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FlexValueVector3 {
    pub m_value: Option<ValueVector3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xff6e8118 {
    pub definition: Unk0x7faa90a0,
    pub transform: Mat4,
    pub name: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapParticle {
    pub group_name: Option<String>,
    pub start_disabled: Option<bool>,
    pub eye_candy: Option<bool>,
    pub m_visibility_flags: Option<u8>,
    pub transform: Mat4,
    pub name: String,
    pub transitional: Option<bool>,
    pub visibility_controller: Option<u32>,
    pub color_modulate: Option<Vec4>,
    pub system: u32,
    pub visibility_mode: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SelectorClipData {
    pub m_flags: Option<u32>,
    pub m_selector_pair_data_list: Vec<SelectorPairData>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9d9f60d2 {
    pub skin: String,
    pub unk_0xde46f1d8: Option<String>,
    pub unk_0xdbde2288: Option<Vec<Unk0x82cab1b3>>,
    pub unk_0x397fe037: Option<bool>,
    pub team: Option<u32>,
    pub r#type: Option<u16>,
    pub unk_0xf1d3a034: Option<bool>,
    pub character_record: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionCharacterMetadata {
    pub m_category: String,
    pub m_data: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxLingerDefinitionData {
    pub separate_linger_color: Option<ValueColor>,
    pub use_separate_linger_color: Option<bool>,
    pub use_linger_scale: Option<bool>,
    pub linger_scale: Option<ValueVector3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x429a2180 {
    pub team: u32,
    pub stop_spawn_time_secs: Option<f32>,
    pub reveal_event: Option<u16>,
    pub camp_name: String,
    pub minimap_icon_offset: Option<Vec3>,
    pub scoreboard_timer: Option<u16>,
    pub unk_0x7d27af7f: Option<bool>,
    pub camp_level: Option<u16>,
    pub tags: Option<Vec<u32>>,
    pub minimap_icon: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ColorChooserMaterialDriver {
    pub m_bool_driver: ColorChooserMaterialDriverMBoolDriver,
    pub m_color_on: Vec4,
    pub m_color_off: Vec4,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf775806c {
    pub skin: String,
    pub unk_0xd1318f26: f32,
    pub team: Option<u32>,
    pub unk_0xf908963: Vec3,
    pub character_record: String,
    pub unk_0x651de225: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConformToPathRigPoseModifierData {
    pub m_ending_joint_name: u32,
    pub m_default_mask_name: u32,
    pub m_starting_joint_name: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxAssetRemap {
    pub new_asset: String,
    pub old_asset: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapLocator {
    pub transform: Mat4,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedVector2fVariableData {
    pub values: Vec<Vec2>,
    pub times: Vec<f32>,
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueFloat {
    pub dynamics: VfxAnimatedFloatVariableData,
    pub constant_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapCubemapProbe {
    pub name: String,
    pub cubemap_probe_scale: f32,
    pub cubemap_probe_path: String,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FlexValueFloat {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxReflectionDefinitionData {
    pub fresnel: Option<f32>,
    pub reflection_fresnel: Option<f32>,
    pub reflection_map_texture: Option<String>,
    pub reflection_opacity_glancing: Option<f32>,
    pub fresnel_color: Option<Vec4>,
    pub reflection_opacity_direct: Option<f32>,
    pub reflection_fresnel_color: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FloatPerSpellLevel {
    pub m_per_level_values: Vec<f32>,
    pub m_value_type: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapAudio {
    pub name: String,
    pub event_name: String,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxTextureMultDefinitionData {
    pub birth_uv_rotate_rate_mult: Option<ValueFloat>,
    pub uv_scroll_clamp_mult: Option<bool>,
    pub uv_transform_center_mult: Option<Vec2>,
    pub emitter_uv_scroll_rate_mult: Option<Vec2>,
    pub is_random_start_frame_mult: Option<bool>,
    pub tex_div_mult: Option<Vec2>,
    pub uv_rotation_mult: Option<ValueFloat>,
    pub birth_uv_offset_mult: Option<ValueVector2>,
    pub birth_uv_scroll_rate_mult: Option<ValueVector2>,
    pub particle_integrated_uv_scroll_mult: Option<IntegratedValueVector2>,
    pub texture_mult_filp_v: Option<bool>,
    pub particle_integrated_uv_rotate_mult: Option<IntegratedValueFloat>,
    pub uv_scroll_alpha_mult: Option<bool>,
    pub uv_scale_mult: Option<ValueVector2>,
    pub tex_address_mode_mult: Option<u8>,
    pub texture_mult: Option<String>,
    pub flex_birth_uv_scroll_rate_mult: Option<FlexValueVector2>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xfcb92181 {
    pub team: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x82cab1b3 {
    pub lane: u16,
    pub position: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FixedSpeedMovement {
    pub m_use_height_offset_at_end: Option<bool>,
    pub m_target_height_augment: Option<f32>,
    pub m_tracks_target: Option<bool>,
    pub m_offset_initial_target_height: Option<f32>,
    pub m_target_bone_name: Option<String>,
    pub m_start_bone_name: Option<String>,
    pub m_speed: f32,
    pub m_project_target_to_cast_range: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SkinCharacterMetaDataProperties {
    pub spawning_skin_offsets: Option<Vec<SkinCharacterMetaDataPropertiesSpawningSkinOffset>>,
    pub relative_color_swap_table: Option<Vec<i32>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EsportsBannerData {
    pub team: u32,
    pub banner_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StatByCoefficientCalculationPart {
    pub m_stat_formula: Option<u8>,
    pub m_stat: Option<u8>,
    pub m_coefficient: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BarracksMinionConfig {
    pub minion_type: u8,
    pub wave_behavior: BarracksMinionConfigWaveBehavior,
    pub minion_upgrade_stats: MinionUpgradeConfig,
    pub unk_0x8a3fc6eb: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapTerrainPaint {
    pub terrain_paint_texture_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SineMaterialDriver {
    pub m_frequency: f32,
    pub m_driver: TimeMaterialDriver,
    pub m_bias: f32,
    pub m_scale: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceListElement {
    pub r#type: String,
    pub name_override: Option<String>,
    pub type_index: Option<i32>,
    pub multiplier: Option<f32>,
    pub style: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapContainer {
    pub convert_streams_to_half_float: bool,
    pub chunks: HashMap<u32, u32>,
    pub bounds_max: Vec2,
    pub components: Vec<MapContainerComponents>,
    pub lowest_walkable_height: f32,
    pub map_path: String,
    pub mesh_combine_radius: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnableLookAtEventData {
    pub m_end_frame: f32,
    pub m_lock_current_values: Option<bool>,
    pub m_enable_look_at: Option<bool>,
    pub m_name: u32,
    pub m_start_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InhibitorWaveBehavior {
    pub spawn_count_per_inhibitor_down: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxTrailDefinitionData {
    pub m_birth_tiling_size: ValueVector3,
    pub m_mode: Option<u8>,
    pub m_max_added_per_frame: Option<i32>,
    pub m_cutoff: Option<f32>,
    pub m_smoothing_mode: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttackSlotData {
    pub m_attack_probability: Option<f32>,
    pub m_attack_cast_time: Option<f32>,
    pub m_attack_delay_cast_offset_percent: Option<f32>,
    pub m_attack_total_time: Option<f32>,
    pub m_attack_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataValue {
    pub m_name: String,
    pub m_values: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionLine {
    pub line_stops_at_end_position: Option<bool>,
    pub end_locator: DrawablePositionLocator,
    pub line_width: FloatPerSpellLevel,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterAudio {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContextualActionCooldownModifications {
    pub dont_reset_timer: Option<bool>,
    pub ignore_timer: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimeBlendData {
    pub m_time: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x8ad25772 {
    pub system: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UseableData {
    pub flags: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xd82714cc {
    pub name: u32,
    pub color: Option<[u8; 4]>,
    pub flags: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SequencerClipData {
    pub m_event_data_map: Option<HashMap<u32, SequencerClipDataMEventDataMap>>,
    pub m_flags: Option<u32>,
    pub m_clip_name_list: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveMesh {
    pub m_mesh: VfxMeshDefinitionData,
    pub align_yaw_to_camera: Option<bool>,
    pub unk_0x6aec9e7a: Option<bool>,
    pub align_pitch_to_camera: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x7ad3dda {
    pub unk_0xbbe68da1: bool,
    pub definition: Unk0x8ad25772,
    pub name: u32,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xb09016f6 {
    pub effect_calculation: GameCalculation,
    pub effect_tag: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ToolSoundData {
    pub death: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapGroup {
    pub name: String,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MinionUpgradeConfig {
    pub unk_0x726ae049: Option<f32>,
    pub armor_upgrade_growth: Option<f32>,
    pub hp_upgrade: f32,
    pub damage_upgrade_late: Option<f32>,
    pub damage_max: f32,
    pub damage_upgrade: Option<f32>,
    pub armor_max: Option<f32>,
    pub gold_max: Option<f32>,
    pub hp_upgrade_late: Option<f32>,
    pub hp_max_bonus: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SkinAudioProperties {
    pub bank_units: Vec<BankUnit>,
    pub tag_event_list: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DynamicMaterialDef {
    pub parameters: Vec<DynamicMaterialParameterDef>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimedWaveBehaviorInfo {
    pub behavior: TimedWaveBehaviorInfoBehavior,
    pub start_time_secs: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SkinCharacterMetaDataPropertiesSpawningSkinOffset {
    pub offset: i32,
    pub tag: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnimationGraphData {
    pub m_cascade_blend_value: f32,
    pub m_sync_group_data_map: Option<HashMap<u32, AnimationGraphDataMSyncGroupDataMap>>,
    pub m_track_data_map: HashMap<u32, AnimationGraphDataMTrackDataMap>,
    pub m_blend_data_table: HashMap<u64, AnimationGraphDataMBlendDataTable>,
    pub m_mask_data_map: Option<HashMap<u32, AnimationGraphDataMMaskDataMap>>,
    pub m_clip_data_map: HashMap<u32, AnimationGraphDataMClipDataMap>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x6355dd6f {
    pub chunk: u32,
    pub visibility_controller: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MaskData {
    pub mid: Option<u32>,
    pub m_weight_list: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LerpMaterialDriver {
    pub m_turn_off_time_sec: f32,
    pub m_on_value: Option<f32>,
    pub m_off_value: Option<f32>,
    pub m_turn_on_time_sec: f32,
    pub m_bool_driver: LerpMaterialDriverMBoolDriver,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceList {
    pub level_count: u32,
    pub elements: Vec<TooltipInstanceListElement>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IsAnimationPlayingDynamicMaterialBoolDriver {
    pub m_animation_names: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContextualSituation {
    pub m_cool_down_time: Option<f32>,
    pub m_rules: Vec<ContextualRule>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JointSnapEventData {
    pub m_joint_name_to_override: u32,
    pub m_joint_name_to_snap_to: u32,
    pub m_start_frame: f32,
    pub m_end_frame: f32,
    pub m_name: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DrawablePositionLocator {
    pub base_position: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedColorVariableData {
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub times: Vec<f32>,
    pub values: Vec<Vec4>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AbilityResourceSlotInfo {
    pub start_at_zero_on_spawn: Option<bool>,
    pub ar_type: Option<u8>,
    pub ar_max_segments: Option<i32>,
    pub ar_base: Option<f32>,
    pub ar_increments: Option<f32>,
    pub ar_has_regen_text: Option<bool>,
    pub ar_is_shown: Option<bool>,
    pub ar_base_static_regen: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeSphere {
    pub radius: Option<f32>,
    pub flags: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeLegacy {
    pub emit_rotation_axes: Option<Vec<Vec3>>,
    pub emit_offset: Option<ValueVector3>,
    pub emit_rotation_angles: Option<Vec<ValueFloat>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapScriptLocator {
    pub name: String,
    pub script_name: String,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NamedDataValueCalculationPart {
    pub m_data_value: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BankUnit {
    pub events: Option<Vec<String>>,
    pub name: String,
    pub bank_path: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlendingSwitchMaterialDriver {
    pub m_default_value: FloatLiteralMaterialDriver,
    pub m_elements: Vec<SwitchMaterialDriverElement>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxProjectionDefinitionData {
    pub m_fading: Option<f32>,
    pub m_y_range: f32,
    pub color_modulate: ValueColor,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FloatGraphMaterialDriver {
    pub graph: VfxAnimatedFloatVariableData,
    pub driver: LerpMaterialDriver,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TrackData {
    pub m_priority: Option<u8>,
    pub m_blend_mode: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MissileSpecification {
    pub m_missile_width: Option<f32>,
    pub movement_component: MissileSpecificationMovementComponent,
    pub height_solver: Option<MissileSpecificationHeightSolver>,
    pub vertical_facing: MissileSpecificationVerticalFacing,
    pub behaviors: Vec<MissileSpecificationBehaviors>,
    pub visibility_component: Option<MissileSpecificationVisibilityComponent>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionCharacter {
    pub m_character_type: u8,
    pub m_child_conditions: Vec<ContextualConditionCharacterMChildConditions>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataResource {
    pub cast_range_display_override: Option<Vec<f32>>,
    pub m_float_vars_decimals: Option<Vec<i32>>,
    pub cast_range: Option<Vec<f32>>,
    pub m_hit_effect_orient_type: Option<u32>,
    pub selection_priority: Option<u32>,
    pub m_line_width: Option<f32>,
    pub m_hit_effect_name: Option<String>,
    pub m_affects_status_flags: Option<u32>,
    pub cast_cone_distance: Option<f32>,
    pub use_animator_framerate: Option<bool>,
    pub m_missile_effect_key: Option<u32>,
    pub m_spell_reveals_champion: Option<bool>,
    pub always_snap_facing: Option<bool>,
    pub m_coefficient2: Option<f32>,
    pub m_considered_as_auto_attack: Option<bool>,
    pub b_have_hit_effect: Option<bool>,
    pub m_cast_time: Option<f32>,
    pub mana: Option<Vec<f32>>,
    pub m_effect_amount: Option<Vec<SpellEffectAmount>>,
    pub missile_speed: Option<f32>,
    pub m_animation_name: Option<String>,
    pub m_update_rotation_when_casting: Option<bool>,
    pub m_client_data: Option<SpellDataResourceClient>,
    pub m_channel_duration: Option<Vec<f32>>,
    pub m_minimap_icon_rotation: Option<bool>,
    pub cant_cast_while_rooted: Option<bool>,
    pub m_ignore_range_check: Option<bool>,
    pub m_show_channel_bar: Option<bool>,
    pub m_spell_tags: Option<Vec<String>>,
    pub cooldown_time: Option<Vec<f32>>,
    pub cast_cone_angle: Option<f32>,
    pub m_alternate_name: Option<String>,
    pub lua_on_missile_update_distance_interval: Option<f32>,
    pub m_do_not_need_to_face_target: Option<bool>,
    pub m_hit_bone_name: Option<String>,
    pub m_dimension_behavior: Option<u8>,
    pub m_targeting_type_data: Option<SpellDataResourceMTargetingTypeData>,
    pub m_missile_spec: Option<MissileSpecification>,
    pub m_use_minimap_targeting: Option<bool>,
    pub m_affects_type_flags: Option<u32>,
    pub m_channel_is_interrupted_by_disables: Option<bool>,
    pub m_casting_breaks_stealth: Option<bool>,
    pub m_project_target_to_cast_range: Option<bool>,
    pub m_doesnt_break_channels: Option<bool>,
    pub m_is_disabled_while_dead: Option<bool>,
    pub m_cost_always_shown_in_ui: Option<bool>,
    pub m_cant_cancel_while_channeling: Option<bool>,
    pub m_cast_type: Option<u32>,
    pub m_use_autoattack_cast_time_data: Option<UseAutoattackCastTimeData>,
    pub m_img_icon_name: Option<Vec<String>>,
    pub data_values_mode_override: Option<HashMap<u32, SpellDataResourceDataValuesModeOverride>>,
    pub delay_total_time_percent: Option<f32>,
    pub cast_frame: Option<f32>,
    pub m_coefficient: Option<f32>,
    pub can_cast_while_disabled: Option<bool>,
    pub m_missile_effect_player_key: Option<u32>,
    pub flags: Option<u32>,
    pub m_particle_start_offset: Option<Vec3>,
    pub m_data_values: Option<Vec<SpellDataValue>>,
    pub m_hit_effect_key: Option<u32>,
    pub m_missile_effect_name: Option<String>,
    pub m_spell_calculations: Option<HashMap<u32, SpellDataResourceMSpellCalculations>>,
    pub m_cant_cancel_while_winding_up: Option<bool>,
    pub b_have_hit_bone: Option<bool>,
    pub m_apply_material_on_hit_sound: Option<bool>,
    pub delay_cast_offset_percent: Option<f32>,
    pub m_minimap_icon_display_flag: Option<u16>,
    pub cast_radius: Option<Vec<f32>>,
    pub m_after_effect_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResourceResolver {
    pub resource_map: Option<HashMap<u32, u32>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValueVector2 {
    pub constant_value: Option<Vec2>,
    pub dynamics: Option<VfxAnimatedVector2fVariableData>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameCalculation {
    pub m_formula_parts: Vec<GameCalculationMFormulaParts>,
    pub m_simple_tooltip_calculation_display: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialShaderParamDef {
    pub name: String,
    pub value: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BuffData {
    pub m_description: Option<String>,
    pub m_tooltip_data: Option<TooltipInstanceBuff>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxFlexShapeDefinitionData {
    pub flex_birth_translation: Option<FlexValueVector3>,
    pub scale_emit_offset_by_bound_object_size: Option<f32>,
    pub flex_scale_emit_offset: Option<FlexTypeFloat>,
    pub scale_birth_translation_by_bound_object_size: Option<f32>,
    pub scale_birth_scale_by_bound_object_size: Option<f32>,
    pub scale_emit_offset_by_bound_object_height: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxMaterialOverrideDefinitionData {
    pub sub_mesh_name: Option<String>,
    pub priority: Option<i32>,
    pub base_texture: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NavGridTerrainConfig {
    pub tags: Vec<Unk0xd82714cc>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FadeEventData {
    pub m_target_alpha: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xfde6a2d7 {
    pub unk_0xdbde2288: Vec<Unk0x82cab1b3>,
    pub barracks_config: u32,
    pub team: Option<u32>,
    pub unk_0xdb6ea1a7: Option<u32>,
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
pub struct TargeterDefinitionAoe {
    pub center_locator: DrawablePositionLocator,
    pub texture_radius_override_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GravityHeightSolver {
    pub m_gravity: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeCylinder {
    pub flags: Option<u8>,
    pub height: Option<f32>,
    pub radius: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldOrbitalDefinitionData {
    pub is_local_space: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x42239bf8 {
    pub transform: Mat4,
    pub definition: Unk0x429a2180,
    pub name: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SkinMeshDataProperties {
    pub bounding_cylinder_radius: Option<f32>,
    pub skeleton: Option<String>,
    pub rig_pose_modifier_data: Option<Vec<ConformToPathRigPoseModifierData>>,
    pub emissive_texture: Option<String>,
    pub fresnel_color: Option<[u8; 4]>,
    pub override_bounding_box: Option<Vec3>,
    pub self_illumination: f32,
    pub reflection_fresnel: Option<f32>,
    pub gloss_texture: Option<String>,
    pub material_override: Option<Vec<SkinMeshDataPropertiesMaterialOverride>>,
    pub submesh_render_order: Option<String>,
    pub reflection_map: Option<String>,
    pub fresnel: Option<f32>,
    pub texture: Option<String>,
    pub reduced_bone_skinning: Option<bool>,
    pub reflection_opacity_glancing: Option<f32>,
    pub initial_submesh_to_hide: Option<String>,
    pub material: Option<u32>,
    pub reflection_opacity_direct: Option<f32>,
    pub skin_scale: Option<f32>,
    pub reflection_fresnel_color: Option<[u8; 4]>,
    pub brush_alpha_override: Option<f32>,
    pub cast_shadows: Option<bool>,
    pub simple_skin: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapBakeProperties {
    pub light_grid_size: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialShaderSamplerDef {
    pub texture_name: String,
    pub address_v: Option<u32>,
    pub texture_path: String,
    pub address_w: Option<u32>,
    pub address_u: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxParentInheritanceParams {
    pub mode: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GearSkinUpgrade {
    pub m_gear_data: GearData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveCameraTrail {
    pub m_trail: VfxTrailDefinitionData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParametricPairData {
    pub m_value: Option<f32>,
    pub m_clip_name: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SkinUpgradeData {
    pub m_gear_skin_upgrades: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BarracksConfig {
    pub move_speed_increase_interval_secs: f32,
    pub upgrades_before_late_game_scaling: i32,
    pub initial_spawn_time_secs: f32,
    pub exp_radius: f32,
    pub upgrade_interval_secs: f32,
    pub units: Vec<BarracksMinionConfig>,
    pub gold_radius: f32,
    pub move_speed_increase_increment: i32,
    pub move_speed_increase_max_times: i32,
    pub minion_spawn_interval_secs: f32,
    pub wave_spawn_interval_secs: f32,
    pub move_speed_increase_initial_delay_secs: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CharacterToolData {
    pub base_attack_speed_bonus: Option<f32>,
    pub map_ai_presence: HashMap<u32, CharacterToolDataMapAiPresence>,
    pub attack_speed: Option<f32>,
    pub description: Option<String>,
    pub chasing_attack_range_percent: Option<f32>,
    pub post_attack_move_delay: Option<f32>,
    pub sound: Option<ToolSoundData>,
    pub soul_given_on_death: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SubmeshVisibilityEventData {
    pub m_name: Option<u32>,
    pub m_hide_submesh_list: Option<Vec<u32>>,
    pub m_show_submesh_list: Option<Vec<u32>>,
    pub m_start_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmissionSurfaceData {
    pub use_surface_normal_for_birth_physics: Option<bool>,
    pub submeshes: Option<Vec<u32>>,
    pub mesh_name: Option<String>,
    pub use_avatar_pose: Option<bool>,
    pub animation_name: Option<String>,
    pub skeleton_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValueFloat {
    pub dynamics: Option<VfxAnimatedFloatVariableData>,
    pub constant_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x111a9fcc {
    pub name: u32,
    pub definition: Unk0xf775806c,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MutatorMapVisibilityController {
    pub mutator_name: String,
    pub path_hash: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxSystemDefinitionData {
    pub particle_path: String,
    pub particle_name: String,
    pub transform: Option<Mat4>,
    pub simple_emitter_definition_data: Option<Vec<VfxEmitterDefinitionData>>,
    pub hud_anchor_position_from_world_projection: Option<bool>,
    pub sound_persistent_default: Option<String>,
    pub complex_emitter_definition_data: Option<Vec<VfxEmitterDefinitionData>>,
    pub flags: Option<u16>,
    pub override_scale_cap: Option<f32>,
    pub sound_on_create_default: Option<String>,
    pub visibility_radius: Option<f32>,
    pub asset_remapping_table: Option<Vec<VfxAssetRemap>>,
    pub build_up_time: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterLegacySimple {
    pub birth_scale: ValueFloat,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveBeam {
    pub m_mesh: Option<VfxMeshDefinitionData>,
    pub m_beam: Option<VfxBeamDefinitionData>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x7faa90a0 {
    pub idle_animation_name: String,
    pub play_idle_animation: Option<bool>,
    pub skin: String,
    pub character_record: String,
    pub team: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameCalculationModified {
    pub m_multiplier: NamedDataValueCalculationPart,
    pub m_modified_game_calculation: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxDistortionDefinitionData {
    pub distortion_mode: Option<u8>,
    pub normal_map_texture: String,
    pub distortion: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedFloatVariableData {
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub times: Vec<f32>,
    pub values: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValueVector3 {
    pub constant_value: Option<Vec3>,
    pub dynamics: Option<VfxAnimatedVector3fVariableData>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveAttachedMesh {
    pub use_avatar_specific_submesh_mask: Option<bool>,
    pub m_mesh: Option<VfxMeshDefinitionData>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeBox {
    pub size: Option<Vec3>,
    pub flags: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContextualActionPlayVo {
    pub m_wait_timeout: Option<f32>,
    pub m_wait_for_announcer_queue: Option<bool>,
    pub m_enemy_event_name: String,
    pub m_max_occurences: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapPlaceableContainer {
    pub items: HashMap<u32, MapPlaceableContainerItems>,
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
pub struct ContextualConditionCharacterName {
    pub m_characters: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveArbitraryTrail {
    pub m_trail: VfxTrailDefinitionData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEventDataPair {
    pub m_target_bone_name: Option<u32>,
    pub m_bone_name: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueVector2 {
    pub dynamics: VfxAnimatedVector2fVariableData,
    pub constant_value: Option<Vec2>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldAccelerationDefinitionData {
    pub is_local_space: Option<bool>,
    pub acceleration: Option<ValueVector3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf6f4bb5f {
    pub name: String,
    pub color: Option<[u8; 4]>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CharacterHealthBarDataRecord {
    pub hp_per_tick: Option<f32>,
    pub unit_health_bar_style: u8,
    pub attach_to_bone: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataValueVector {
    pub spell_data_values: Vec<SpellDataValue>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AbilityObject {
    pub m_name: String,
    pub m_child_spells: Vec<u32>,
    pub m_root_spell: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xcf4a55da {
    pub overlays: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldCollectionDefinitionData {
    pub field_attraction_definitions: Option<Vec<VfxFieldAttractionDefinitionData>>,
    pub field_noise_definitions: Option<Vec<VfxFieldNoiseDefinitionData>>,
    pub field_acceleration_definitions: Option<Vec<VfxFieldAccelerationDefinitionData>>,
    pub field_orbital_definitions: Option<Vec<VfxFieldOrbitalDefinitionData>>,
    pub field_drag_definitions: Option<Vec<VfxFieldDragDefinitionData>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterFiltering {
    pub spectator_policy: Option<u8>,
    pub keywords_excluded: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FixedTimeMovement {
    pub m_target_height_augment: Option<f32>,
    pub m_target_bone_name: Option<String>,
    pub m_travel_time: f32,
    pub m_offset_initial_target_height: Option<f32>,
    pub m_start_bone_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldAttractionDefinitionData {
    pub position: Option<ValueVector3>,
    pub radius: ValueFloat,
    pub acceleration: ValueFloat,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContextualActionData {
    pub m_cooldown: f32,
    pub m_situations: HashMap<u32, ContextualActionDataMSituations>,
    pub m_object_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpellObject {
    pub bot_data: Option<BotsSpellData>,
    pub object_name: String,
    pub m_script_name: String,
    pub m_buff: Option<BuffData>,
    pub m_spell: Option<SpellDataResource>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SkinCharacterDataProperties {
    pub extra_character_preloads: Option<Vec<String>>,
    pub m_spawn_particle_name: Option<String>,
    pub default_animations: Option<Vec<String>>,
    pub m_additional_resource_resolvers: Option<Vec<u32>>,
    pub skin_upgrade_data: Option<SkinUpgradeData>,
    // pub m_emblems: Option<Vec<SkinEmblem>>,
    pub unk_0xe484edc4: Option<u32>,
    pub skin_mesh_properties: Option<SkinMeshDataProperties>,
    pub icon_circle_scale: Option<f32>,
    pub alternate_icons_circle: Option<Vec<String>>,
    pub armor_material: Option<String>,
    pub particle_override_death_particle: Option<String>,
    pub uncensored_icon_circles: Option<HashMap<u32, String>>,
    // pub unk_0xb67a2dd8: Option<Vec<Unk0x9c1d99c0>>,
    pub hud_mute_event: Option<String>,
    pub uncensored_icon_squares: Option<HashMap<u32, String>>,
    pub skin_audio_properties: Option<SkinAudioProperties>,
    // pub loadscreen: Option<CensoredImage>,
    pub theme_music: Option<Vec<String>>,
    // pub persistent_effect_conditions:
    // Option<Vec<SkinCharacterDataPropertiesPersistentEffectConditions>>,
    pub icon_square: Option<String>,
    pub emote_buffbone: Option<String>,
    pub health_bar_data: Option<CharacterHealthBarDataRecord>,
    pub icon_avatar: Option<String>,
    pub extra_action_button_count: Option<u32>,
    pub attribute_flags: Option<u32>,
    pub particle_override_champion_kill_death_particle: Option<String>,
    pub m_resource_resolver: Option<u32>,
    pub m_contextual_action_data: Option<u32>,
    pub skin_classification: Option<u32>,
    // pub loadscreen_vintage: Option<CensoredImage>,
    pub emote_loadout: Option<u32>,
    pub skin_parent: Option<i32>,
    pub hud_unmute_event: Option<String>,
    pub meta_data_tags: Option<String>,
    pub override_on_screen_name: Option<String>,
    pub emote_y_offset: Option<f32>,
    pub icon_circle: Option<String>,
    pub unk_0xeda7817e: Option<u32>,
    // pub secondary_resource_hud_display_data: Option<SecondaryResourceDisplayFractional>,
    pub idle_particles_effects: Option<Vec<SkinCharacterDataPropertiesCharacterIdleEffect>>,
    pub champion_skin_name: Option<String>,
    pub unk_0x2ac577e2: Option<bool>,
    pub godray_f_xbone: Option<String>,
    pub skin_animation_properties: SkinAnimationProperties,
    pub can_share_theme_music: Option<bool>,
    pub alternate_icons_square: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConditionFloatClipData {
    pub updater: ConditionFloatClipDataUpdater,
    pub m_flags: Option<u32>,
    pub m_change_animation_mid_play: Option<bool>,
    pub m_play_anim_change_from_beginning: Option<bool>,
    pub m_condition_float_pair_data_list: Vec<ConditionFloatPairData>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwitchMaterialDriver {
    pub m_default_value: FloatLiteralMaterialDriver,
    pub m_elements: Vec<SwitchMaterialDriverElement>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionChanceToPlay {
    pub m_percent_chance_to_play: u8,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TargetLaserComponentEffects {
    pub beam_effect_definition: SkinCharacterDataPropertiesCharacterIdleEffect,
    pub champ_targeting_effect_definition: Option<SkinCharacterDataPropertiesCharacterIdleEffect>,
    pub tower_targeting_effect_definition: Option<SkinCharacterDataPropertiesCharacterIdleEffect>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueVector3 {
    pub dynamics: VfxAnimatedVector3fVariableData,
    pub constant_value: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x2bfb084c {
    pub group_name: String,
    pub tags: Vec<Unk0xf6f4bb5f>,
    pub unk_0xec01928c: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConditionFloatPairData {
    pub m_value: Option<f32>,
    pub m_hold_animation_to_lower: Option<f32>,
    pub m_hold_animation_to_higher: Option<f32>,
    pub m_clip_name: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConformToPathEventData {
    pub m_blend_in_time: f32,
    pub m_blend_out_time: f32,
    pub m_mask_data_name: u32,
    pub m_start_frame: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimeMaterialDriver {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FlexValueVector2 {
    pub m_value: ValueVector2,
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
pub struct VfxAnimatedVector3fVariableData {
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub times: Vec<f32>,
    pub values: Vec<Vec3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxChildParticleSetDefinitionData {
    pub parent_inheritance_definition: Option<VfxParentInheritanceParams>,
    pub children_probability: Option<ValueFloat>,
    pub bone_to_spawn_at: Option<Vec<String>>,
    pub children_identifiers: Vec<VfxChildIdentifier>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AtomicClipData {
    pub m_track_data_name: u32,
    pub m_mask_data_name: Option<u32>,
    pub m_tick_duration: Option<f32>,
    pub end_frame: Option<f32>,
    pub m_sync_group_data_name: Option<u32>,
    pub m_animation_resource_data: AnimationResourceData,
    pub start_frame: Option<f32>,
    pub m_event_data_map: Option<HashMap<u32, AtomicClipDataMEventDataMap>>,
    pub m_flags: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SkinAnimationProperties {
    pub animation_graph_data: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xcdb1c8f6 {
    pub unk_0x6355dd6f: Vec<Unk0x6355dd6f>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf3cbe7b2 {
    pub m_spell_calculation_key: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransitionClipBlendData {
    pub m_clip_name: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionRange {
    pub use_caster_bounding_box: Option<bool>,
    pub texture_orientation: Option<u32>,
    pub hide_with_line_indicator: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SoundEventData {
    pub m_name: Option<u32>,
    pub m_is_kill_event: Option<bool>,
    pub m_sound_name: Option<String>,
    pub m_is_loop: Option<bool>,
    pub m_start_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialSwitchDef {
    pub group: Option<String>,
    pub name: String,
    pub on: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitivePlanarProjection {
    pub m_projection: VfxProjectionDefinitionData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialPassDef {
    pub dst_color_blend_factor: Option<u32>,
    pub shader: u32,
    pub cull_enable: Option<bool>,
    pub src_alpha_blend_factor: Option<u32>,
    pub src_color_blend_factor: Option<u32>,
    pub write_mask: Option<u32>,
    pub dst_alpha_blend_factor: Option<u32>,
    pub blend_enable: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialChildTechniqueDef {
    pub name: String,
    pub shader_macros: HashMap<String, String>,
    pub parent_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldNoiseDefinitionData {
    pub position: Option<ValueVector3>,
    pub frequency: ValueFloat,
    pub velocity_delta: ValueFloat,
    pub axis_fraction: Vec3,
    pub radius: ValueFloat,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParallelClipData {
    pub m_clip_name_list: Vec<u32>,
    pub m_flags: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionMinimap {
    pub use_caster_bounding_box: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FollowTerrainHeightSolver {
    pub m_height_offset: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxChildIdentifier {
    pub effect: Option<u32>,
    pub effect_key: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GdsMapObject {
    pub extra_info: Option<Vec<GdsMapObjectBannerInfo>>,
    pub visibility_controller: Option<u32>,
    pub name: String,
    pub box_max: Option<Vec3>,
    pub box_min: Option<Vec3>,
    pub transform: Mat4,
    pub r#type: u8,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UseAutoattackCastTimeData {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xc406a533 {
    pub default_visible: bool,
    pub path_hash: u32,
    pub unk_0x27639032: u8,
    pub name: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NavGridConfig {
    pub terrain_config: u32,
    pub region_groups: Vec<Unk0x2bfb084c>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpellEffectAmount {
    pub value: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxBeamDefinitionData {
    pub m_birth_tiling_size: ValueVector3,
    pub m_animated_color_with_distance: Option<ValueColor>,
    pub m_mode: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x3c995caf {
    pub segments: Vec<Vec3>,
    pub name: String,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceBuff {
    pub m_loc_keys: HashMap<String, String>,
    pub m_format: u32,
    pub m_object_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DynamicMaterialParameterDef {
    pub name: String,
    pub driver: DynamicMaterialParameterDefDriver,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataResourceClient {
    pub m_targeter_definitions: Option<Vec<SpellDataResourceClientMTargeterDefinitions>>,
    pub m_tooltip_data: Option<TooltipInstanceSpell>,
    pub m_use_tooltip_from_another_spell: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEventData {
    pub m_particle_event_data_pair_list: Vec<ParticleEventDataPair>,
    pub m_is_loop: Option<bool>,
    pub m_end_frame: Option<f32>,
    pub m_is_detachable: Option<bool>,
    pub m_effect_name: Option<String>,
    pub m_is_kill_event: Option<bool>,
    pub m_effect_key: Option<u32>,
    pub m_start_frame: Option<f32>,
    pub m_name: Option<u32>,
    pub m_enemy_effect_key: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceSpell {
    pub m_loc_keys: HashMap<String, String>,
    pub m_object_name: String,
    pub m_format: u32,
    pub m_lists: Option<HashMap<String, TooltipInstanceSpellMLists>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SkinMeshDataPropertiesMaterialOverride {
    pub material: Option<u32>,
    pub texture: Option<String>,
    pub submesh: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxMeshDefinitionData {
    pub m_mesh_skeleton_name: Option<String>,
    pub m_submeshes_to_draw: Option<Vec<u32>>,
    pub m_submeshes_to_draw_always: Option<Vec<u32>>,
    pub m_animation_name: Option<String>,
    pub m_lock_mesh_to_attachment: Option<bool>,
    pub m_simple_mesh_name: Option<String>,
    pub m_mesh_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FloatLiteralMaterialDriver {
    pub m_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxAlphaErosionDefinitionData {
    pub erosion_feather_in: Option<f32>,
    pub linger_erosion_drive_curve: Option<ValueFloat>,
    pub erosion_map_name: Option<String>,
    pub erosion_slice_width: Option<f32>,
    pub erosion_map_channel_mixer: Option<ValueColor>,
    pub erosion_drive_curve: Option<ValueFloat>,
    pub erosion_drive_source: Option<u8>,
    pub use_linger_erosion_drive_curve: Option<bool>,
    pub erosion_map_address_mode: Option<u8>,
    pub erosion_feather_out: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterDefinitionData {
    pub importance: Option<u8>,
    pub rotation0: Option<IntegratedValueVector3>,
    pub particle_color_texture: Option<String>,
    pub particle_is_local_orientation: Option<bool>,
    pub is_following_terrain: Option<bool>,
    pub particle_linger: Option<f32>,
    pub linger: Option<VfxLingerDefinitionData>,
    pub mesh_render_flags: Option<u8>,
    pub start_frame: Option<u16>,
    pub birth_scale0: Option<ValueVector3>,
    pub is_uniform_scale: Option<bool>,
    pub disable_backface_cull: Option<bool>,
    pub alpha_erosion_definition: Option<VfxAlphaErosionDefinitionData>,
    pub is_local_orientation: Option<bool>,
    pub direction_velocity_min_scale: Option<f32>,
    pub post_rotate_orientation_axis: Option<Vec3>,
    pub has_variable_start_time: Option<bool>,
    pub texture_flip_v: Option<bool>,
    pub particle_lifetime: Option<ValueFloat>,
    pub birth_drag: Option<ValueVector3>,
    pub audio: Option<VfxEmitterAudio>,
    pub is_direction_oriented: Option<bool>,
    pub reflection_definition: Option<VfxReflectionDefinitionData>,
    pub emission_surface_definition: Option<VfxEmissionSurfaceData>,
    pub uv_rotation: Option<ValueFloat>,
    pub disabled: Option<bool>,
    pub use_emission_mesh_normal_for_birth: Option<bool>,
    pub legacy_simple: Option<VfxEmitterLegacySimple>,
    pub direction_velocity_scale: Option<f32>,
    pub chance_to_not_exist: Option<f32>,
    pub stencil_mode: Option<u8>,
    pub color_look_up_type_y: Option<u8>,
    pub uv_scale: Option<ValueVector2>,
    pub has_post_rotate_orientation: Option<bool>,
    pub rotation_override: Option<Vec3>,
    pub translation_override: Option<Vec3>,
    pub birth_uv_rotate_rate: Option<ValueFloat>,
    pub uv_mode: Option<u8>,
    pub misc_render_flags: Option<u8>,
    pub uv_transform_center: Option<Vec2>,
    pub blend_mode: Option<u8>,
    pub rate_by_velocity_function: Option<ValueVector2>,
    pub is_ground_layer: Option<bool>,
    pub spawn_shape: Option<VfxEmitterDefinitionDataSpawnShape>,
    pub lifetime: Option<f32>,
    pub filtering: Option<VfxEmitterFiltering>,
    pub field_collection_definition: Option<VfxFieldCollectionDefinitionData>,
    pub distortion_definition: Option<VfxDistortionDefinitionData>,
    pub unk_0xcb13aff1: Option<f32>,
    pub world_acceleration: Option<IntegratedValueVector3>,
    pub emitter_name: String,
    pub tex_div: Option<Vec2>,
    pub rate: Option<ValueFloat>,
    pub scale0: Option<ValueVector3>,
    pub birth_acceleration: Option<ValueVector3>,
    pub child_particle_set_definition: Option<VfxChildParticleSetDefinitionData>,
    pub particle_uv_scroll_rate: Option<IntegratedValueVector2>,
    pub velocity: Option<ValueVector3>,
    pub birth_orbital_velocity: Option<ValueVector3>,
    pub num_frames: Option<u16>,
    pub acceleration: Option<ValueVector3>,
    pub palette_definition: Option<VfxPaletteDefinitionData>,
    pub uv_parallax_scale: Option<f32>,
    pub uv_scroll_clamp: Option<bool>,
    pub color: Option<ValueColor>,
    pub stencil_ref: Option<u8>,
    pub emitter_uv_scroll_rate: Option<Vec2>,
    pub birth_uv_offset: Option<ValueVector2>,
    pub particle_uv_rotate_rate: Option<IntegratedValueFloat>,
    pub color_render_flags: Option<u8>,
    pub sort_emitters_by_pos: Option<bool>,
    pub slice_technique_range: Option<f32>,
    pub birth_velocity: Option<ValueVector3>,
    pub flex_instance_scale: Option<FlexTypeFloat>,
    pub color_look_up_scales: Option<Vec2>,
    pub alpha_ref: Option<u8>,
    pub pass: Option<i16>,
    pub color_look_up_offsets: Option<Vec2>,
    pub period: Option<f32>,
    pub birth_uv_scroll_rate: Option<ValueVector2>,
    pub time_active_during_period: Option<f32>,
    pub texture: Option<String>,
    pub texture_flip_u: Option<bool>,
    pub does_cast_shadow: Option<bool>,
    pub birth_color: Option<ValueColor>,
    pub use_navmesh_mask: Option<bool>,
    pub soft_particle_params: Option<VfxSoftParticleDefinitionData>,
    pub texture_mult: Option<VfxTextureMultDefinitionData>,
    pub birth_rotation0: Option<ValueVector3>,
    pub birth_rotational_velocity0: Option<ValueVector3>,
    pub flex_shape_definition: Option<VfxFlexShapeDefinitionData>,
    pub modulation_factor: Option<Vec4>,
    pub flex_particle_lifetime: Option<FlexValueFloat>,
    pub frame_rate: Option<f32>,
    pub falloff_texture: Option<String>,
    pub color_look_up_type_x: Option<u8>,
    pub is_random_start_frame: Option<bool>,
    pub emission_mesh_name: Option<String>,
    pub flex_birth_uv_offset: Option<FlexValueVector2>,
    pub emitter_position: Option<ValueVector3>,
    pub censor_modulate_value: Option<Vec4>,
    pub is_rotation_enabled: Option<bool>,
    pub primitive: Option<VfxEmitterDefinitionDataPrimitive>,
    pub maximum_rate_by_velocity: Option<f32>,
    pub depth_bias_factors: Option<Vec2>,
    pub particles_share_random_value: Option<bool>,
    pub bind_weight: Option<ValueFloat>,
    pub material_override_definitions: Option<Vec<VfxMaterialOverrideDefinitionData>>,
    pub is_single_particle: Option<bool>,
    pub emitter_linger: Option<f32>,
    pub particle_linger_type: Option<u8>,
    pub tex_address_mode_base: Option<u8>,
    pub birth_frame_rate: Option<ValueFloat>,
    pub emission_mesh_scale: Option<f32>,
    pub drag: Option<ValueVector3>,
    pub time_before_first_emission: Option<f32>,
    pub is_emitter_space: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Defaultvisibility {
    pub m_perception_bubble_radius: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AcceleratingMovement {
    pub m_tracks_target: bool,
    pub m_min_speed: f32,
    pub m_offset_initial_target_height: f32,
    pub m_max_speed: f32,
    pub m_initial_speed: f32,
    pub m_acceleration: f32,
    pub m_use_height_offset_at_end: bool,
    pub m_start_bone_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldDragDefinitionData {
    pub strength: Option<ValueFloat>,
    pub position: Option<ValueVector3>,
    pub radius: Option<ValueFloat>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xee39916f {
    pub emit_offset: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VfxProbabilityTableData {
    pub key_times: Option<Vec<f32>>,
    pub key_values: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MapSunProperties {
    pub fog_start_and_end: Vec2,
    pub fog_color: Vec4,
    pub sky_light_scale: f32,
    pub ground_color: Vec4,
    pub fog_alternate_color: Vec4,
    pub sun_direction: Vec3,
    pub sky_light_color: Vec4,
    pub horizon_color: Vec4,
    pub light_map_color_scale: f32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xec733fe2 {
    pub default_visible: Option<bool>,
    pub unk_0x8bff8cdf: u8,
    pub path_hash: u32,
    pub name: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x3c2bf0c0 {
    pub name: u32,
    pub unk_0xbbe68da1: Option<bool>,
    pub transform: Mat4,
    pub definition: Unk0x9d9f60d2,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RotatingWaveBehavior {
    pub spawn_counts_by_wave: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnimationResourceData {
    pub m_animation_file_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContextualRule {
    pub m_rule_name: Option<String>,
    pub cooldown_modifications: Option<ContextualActionCooldownModifications>,
    pub m_audio_action: ContextualActionPlayVo,
    pub m_conditions: Option<Vec<ContextualRuleMConditions>>,
    pub m_priority: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConstantWaveBehavior {
    pub spawn_count: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialDef {
    pub techniques: Vec<StaticMaterialTechniqueDef>,
    pub child_techniques: Option<Vec<StaticMaterialChildTechniqueDef>>,
    pub shader_macros: HashMap<String, String>,
    pub switches: Option<Vec<StaticMaterialSwitchDef>>,
    pub dynamic_material: Option<DynamicMaterialDef>,
    pub sampler_values: Vec<StaticMaterialShaderSamplerDef>,
    pub param_values: Vec<StaticMaterialShaderParamDef>,
    pub r#type: Option<u32>,
    pub name: String,
}
