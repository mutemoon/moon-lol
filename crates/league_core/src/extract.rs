use std::collections::HashMap;

use bevy::{
    asset::Asset,
    math::{Mat4, Vec2, Vec3, Vec4},
    reflect::TypePath,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StatStoneEventToTrackStatFilters {
    TargetHasUnitTagFilter(TargetHasUnitTagFilter),
    TargetTypeFilter(TargetTypeFilter),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ContextualConditionCharacterMChildConditions {
    ContextualConditionCharacterHealth(ContextualConditionCharacterHealth),
    ContextualConditionIsAlly(ContextualConditionIsAlly),
    ContextualConditionCharacterInRangeForSyncedAnimation,
    ContextualConditionIsSelf(ContextualConditionIsSelf),
    ContextualConditionCharacterUnitTags(ContextualConditionCharacterUnitTags),
    ContextualConditionCharacterPlayingAnimation(ContextualConditionCharacterPlayingAnimation),
    ContextualConditionCharacterIsCastingRecall,
    ContextualConditionCharacterPlayingEmote,
    ContextualConditionCharacterDistance(ContextualConditionCharacterDistance),
    ContextualConditionCharacterHasTimeRemainingForAnimation(
        ContextualConditionCharacterHasTimeRemainingForAnimation,
    ),
    ContextualConditionCharacterMetadata(ContextualConditionCharacterMetadata),
    ContextualConditionCharacterName(ContextualConditionCharacterName),
    Unk0xb6da23cb,
    ContextualConditionCharacterSkinId(ContextualConditionCharacterSkinId),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SpellRankUpRequirementsMRequirements {
    CharacterLevelRequirement(CharacterLevelRequirement),
    HasBuffRequirement(HasBuffRequirement),
    HasSkillPointRequirement,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AtomicClipDataMEventDataMap {
    ParticleEventData(ParticleEventData),
    ConformToPathEventData(ConformToPathEventData),
    JointOrientationEventData(JointOrientationEventData),
    IdleParticlesVisibilityEventData(IdleParticlesVisibilityEventData),
    JointSnapEventData(JointSnapEventData),
    EnableLookAtEventData(EnableLookAtEventData),
    SoundEventData(SoundEventData),
    FaceTargetEventData(FaceTargetEventData),
    SubmeshVisibilityEventData(SubmeshVisibilityEventData),
    FadeEventData(FadeEventData),
    SpringPhysicsEventData(SpringPhysicsEventData),
    StopAnimationEventData(StopAnimationEventData),
    LockRootOrientationEventData(LockRootOrientationEventData),
    SyncedAnimationEventData(SyncedAnimationEventData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BlendingSwitchMaterialDriverMDefaultValue {
    IsMovingBoolDriver,
    Unk0x83a9f4f8,
    SwitchMaterialDriver(SwitchMaterialDriver),
    Unk0xfe70e9c4(Unk0xfe70e9c4),
    NotMaterialDriver(NotMaterialDriver),
    VelocityDynamicMaterialFloatDriver,
    BlendingSwitchMaterialDriver(BlendingSwitchMaterialDriver),
    AllTrueMaterialDriver(AllTrueMaterialDriver),
    MaxMaterialDriver(MaxMaterialDriver),
    IsEnemyDynamicMaterialBoolDriver,
    FixedDurationTriggeredBoolDriver(FixedDurationTriggeredBoolDriver),
    FloatGraphMaterialDriver(FloatGraphMaterialDriver),
    IsDeadDynamicMaterialBoolDriver,
    OneTrueMaterialDriver(OneTrueMaterialDriver),
    Float4LiteralMaterialDriver(Float4LiteralMaterialDriver),
    HasBuffOfTypeBoolDriver(HasBuffOfTypeBoolDriver),
    Unk0xf5821f8b,
    Unk0x5b2fdd66(Unk0x5b2fdd66),
    HasBuffWithAttributeBoolDriver,
    ColorGraphMaterialDriver(ColorGraphMaterialDriver),
    RemapFloatMaterialDriver(RemapFloatMaterialDriver),
    HasGearDynamicMaterialBoolDriver(HasGearDynamicMaterialBoolDriver),
    BuffCounterDynamicMaterialFloatDriver(BuffCounterDynamicMaterialFloatDriver),
    Unk0xb7b43e1d(Unk0xb7b43e1d),
    IsInGrassDynamicMaterialBoolDriver,
    SubmeshVisibilityBoolDriver(SubmeshVisibilityBoolDriver),
    HasBuffDynamicMaterialBoolDriver(HasBuffDynamicMaterialBoolDriver),
    LerpVec4LogicDriver(LerpVec4LogicDriver),
    PlayerPositionDynamicMaterialDriver,
    Unk0x635d04b7(Unk0x635d04b7),
    UvScaleBiasFromAnimationDynamicMaterialDriver(UvScaleBiasFromAnimationDynamicMaterialDriver),
    DistanceToPlayerMaterialFloatDriver(DistanceToPlayerMaterialFloatDriver),
    SpecificColorMaterialDriver(SpecificColorMaterialDriver),
    Unk0x9bc366ca(Unk0x9bc366ca),
    MinMaterialDriver(MinMaterialDriver),
    LerpMaterialDriver(LerpMaterialDriver),
    RemapVec4MaterialDriver(RemapVec4MaterialDriver),
    Unk0x77b42f3f,
    IsAnimationPlayingDynamicMaterialBoolDriver(IsAnimationPlayingDynamicMaterialBoolDriver),
    SpellRankIntDriver(SpellRankIntDriver),
    KeyFrameFloatClipReaderDriver(KeyFrameFloatClipReaderDriver),
    TimeMaterialDriver(TimeMaterialDriver),
    HealthDynamicMaterialFloatDriver,
    SineMaterialDriver(SineMaterialDriver),
    IsCastingBoolDriver(IsCastingBoolDriver),
    AnimationFractionDynamicMaterialFloatDriver(AnimationFractionDynamicMaterialFloatDriver),
    FloatComparisonMaterialDriver(FloatComparisonMaterialDriver),
    IsLocalPlayerBoolDriver,
    IsAttackingBoolDriver,
    ColorChooserMaterialDriver(ColorChooserMaterialDriver),
    FloatLiteralMaterialDriver(FloatLiteralMaterialDriver),
    DelayedBoolMaterialDriver(DelayedBoolMaterialDriver),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TargetingParametersRangeValue {
    Unk0x9d62f7e(Unk0x9d62f7e),
    TargetingRangeValue(TargetingRangeValue),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StatBySubPartCalculationPartMSubpart {
    ProductOfSubPartsCalculationPart(ProductOfSubPartsCalculationPart),
    PercentageOfBuffNameElapsed(PercentageOfBuffNameElapsed),
    NamedDataValueCalculationPart(NamedDataValueCalculationPart),
    Unk0x8a96ea3c(Unk0x8a96ea3c),
    CooldownMultiplierCalculationPart,
    EffectValueCalculationPart(EffectValueCalculationPart),
    StatByNamedDataValueCalculationPart(StatByNamedDataValueCalculationPart),
    ByCharLevelBreakpointsCalculationPart(ByCharLevelBreakpointsCalculationPart),
    Unk0xf3cbe7b2(Unk0xf3cbe7b2),
    Unk0xee18a47b(Unk0xee18a47b),
    BuffCounterByCoefficientCalculationPart(BuffCounterByCoefficientCalculationPart),
    AbilityResourceByCoefficientCalculationPart(AbilityResourceByCoefficientCalculationPart),
    StatBySubPartCalculationPart(StatBySubPartCalculationPart),
    ByCharLevelFormulaCalculationPart(ByCharLevelFormulaCalculationPart),
    BuffCounterByNamedDataValueCalculationPart(BuffCounterByNamedDataValueCalculationPart),
    Unk0x1d452085(Unk0x1d452085),
    ByCharLevelInterpolationCalculationPart(ByCharLevelInterpolationCalculationPart),
    StatByCoefficientCalculationPart(StatByCoefficientCalculationPart),
    NumberCalculationPart(NumberCalculationPart),
    ClampSubPartsCalculationPart(ClampSubPartsCalculationPart),
    Unk0x9e9e2e5c(Unk0x9e9e2e5c),
    SumOfSubPartsCalculationPart(SumOfSubPartsCalculationPart),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TriggerOnDelayMActions {
    ChangeMissileSpeed(ChangeMissileSpeed),
    Cast(Cast),
    Destroy,
    CallOnMissileBounce,
    ClearTargetAndKeepMoving(ClearTargetAndKeepMoving),
    ClearAlreadyHitTracking,
    ChangeMissileWidth(ChangeMissileWidth),
    AttackEvents(AttackEvents),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CustomTargeterDefinitionsMTargeterDefinitions {
    TargeterDefinitionMultiAoe(TargeterDefinitionMultiAoe),
    TargeterDefinitionRange(TargeterDefinitionRange),
    TargeterDefinitionCone(TargeterDefinitionCone),
    TargeterDefinitionMinimap(TargeterDefinitionMinimap),
    TargeterDefinitionWall(TargeterDefinitionWall),
    TargeterDefinitionAoe(TargeterDefinitionAoe),
    TargeterDefinitionLine(TargeterDefinitionLine),
    TargeterDefinitionArc(TargeterDefinitionArc),
    TargeterDefinitionSpline(TargeterDefinitionSpline),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AnimationGraphDataMBlendDataTable {
    TimeBlendData(TimeBlendData),
    TransitionClipBlendData(TransitionClipBlendData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AnimationGraphDataMClipDataMap {
    SequencerClipData(SequencerClipData),
    ConditionFloatClipData(ConditionFloatClipData),
    ParametricClipData(ParametricClipData),
    SelectorClipData(SelectorClipData),
    ParallelClipData(ParallelClipData),
    ConditionBoolClipData(ConditionBoolClipData),
    AtomicClipData(AtomicClipData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SpellDataResourceUnknownEnumField {
    Unk0xf00f3333,
    Unk0x6bbc3db6(Unk0x6bbc3db6),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SpellDataResourceMTargetingTypeData {
    Area,
    SelfAoe,
    Target(Target),
    TargetOrLocation,
    Location,
    Direction,
    MySelf,
    Cone,
    AreaClamped,
    TerrainType(TerrainType),
    LocationClamped,
    WallDetection(WallDetection),
    DragDirection,
    TerrainLocation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SpellDataResourceMCastRequirementsCaster {
    IsSpecifiedUnitCastRequirement(IsSpecifiedUnitCastRequirement),
    SameTeamCastRequirement(SameTeamCastRequirement),
    HasNNearbyUnitsRequirement(HasNNearbyUnitsRequirement),
    HasBuffCastRequirement(HasBuffCastRequirement),
    HasNNearbyVisibleUnitsRequirement(HasNNearbyVisibleUnitsRequirement),
    HasAtleastNSubRequirementsCastRequirement(HasAtleastNSubRequirementsCastRequirement),
    HasTypeAndStatusFlags(HasTypeAndStatusFlags),
    HasAllSubRequirementsCastRequirement(HasAllSubRequirementsCastRequirement),
    HasUnitTagsCastRequirement(HasUnitTagsCastRequirement),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SpellDataResourceMSpellCalculations {
    GameCalculation(GameCalculation),
    GameCalculationModified(GameCalculationModified),
    GameCalculationConditional(GameCalculationConditional),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MissileSpecificationVerticalFacing {
    VeritcalFacingMatchVelocity,
    VerticalFacingFaceTarget,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MissileSpecificationHeightSolver {
    FollowTerrainHeightSolver(FollowTerrainHeightSolver),
    CurveTheDifferenceHeightSolver(CurveTheDifferenceHeightSolver),
    BlendedLinearHeightSolver,
    SinusoidalHeightSolver(SinusoidalHeightSolver),
    GravityHeightSolver(GravityHeightSolver),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MissileSpecificationVisibilityComponent {
    Defaultvisibility(Defaultvisibility),
    EnterFowVisibility(EnterFowVisibility),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MissileSpecificationBehaviors {
    DestroyOnMovementComplete(DestroyOnMovementComplete),
    WidthPerSecond(WidthPerSecond),
    Unk0x72f86c81(Unk0x72f86c81),
    MapLocator(MapLocator),
    MapTerrainPaint(MapTerrainPaint),
    MapParticle(MapParticle),
    Unk0x3c995caf(Unk0x3c995caf),
    MapSunProperties(MapSunProperties),
    DestroyOnHit,
    Unk0xba138ae3(Unk0xba138ae3),
    PersistentEffectConditionData(PersistentEffectConditionData),
    Unk0x9aa5b4bc(Unk0x9aa5b4bc),
    Unk0x25e3f5d0(Unk0x25e3f5d0),
    Unk0x0,
    Unk0x1f1f50f2(Unk0x1f1f50f2),
    ReturnToCasterOnMovementComplete(ReturnToCasterOnMovementComplete),
    Unk0xd178749c(Unk0xd178749c),
    MapBakeProperties(MapBakeProperties),
    TriggerOnHit(TriggerOnHit),
    Unk0xcdb1c8f6(Unk0xcdb1c8f6),
    MapScriptLocator(MapScriptLocator),
    Unk0xcf4a55da(Unk0xcf4a55da),
    TriggerFromScript(TriggerFromScript),
    DelayStart(DelayStart),
    MapCubemapProbe(MapCubemapProbe),
    TriggerOnMovementComplete(TriggerOnMovementComplete),
    CastOnMovementComplete(CastOnMovementComplete),
    CastOnHit(CastOnHit),
    ResimulateTrailVfxOnEnterVisibility(ResimulateTrailVfxOnEnterVisibility),
    MapNavGrid(MapNavGrid),
    Unk0xeb997689(Unk0xeb997689),
    Unk0xad65d8c4(Unk0xad65d8c4),
    TriggerOnDelay(TriggerOnDelay),
    FixedDistanceIgnoringTerrain(FixedDistanceIgnoringTerrain),
    MapAudio(MapAudio),
    GdsMapObject(GdsMapObject),
    MapGroup(MapGroup),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MissileSpecificationMovementComponent {
    FixedSpeedMovement(FixedSpeedMovement),
    TrackMouseMovement(TrackMouseMovement),
    WallFollowMovement(WallFollowMovement),
    FixedTimeMovement(FixedTimeMovement),
    FixedSpeedSplineMovement(FixedSpeedSplineMovement),
    DecelToLocationMovement(DecelToLocationMovement),
    PhysicsMovement(PhysicsMovement),
    AcceleratingMovement(AcceleratingMovement),
    ParametricMovement(ParametricMovement),
    FixedTimeSplineMovement(FixedTimeSplineMovement),
    SyncCircleMovement(SyncCircleMovement),
    CircleMovement(CircleMovement),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BasicSkinAugmentModifiers {
    Unk0x9c5b78dd(Unk0x9c5b78dd),
    Unk0xeea0bf1,
    Unk0x51ada002(Unk0x51ada002),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ScriptDataObjectMConstants {
    GameModeConstantInteger(GameModeConstantInteger),
    GameModeConstantString(GameModeConstantString),
    GameModeConstantStringVector(GameModeConstantStringVector),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TargeterDefinitionLineIndicatorType {
    IndicatorTypeLocal,
    IndicatorTypeGlobal(IndicatorTypeGlobal),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PackManagerDataUnknownEnumField {
    Unk0x1ddfbeeb,
    Unk0x1aae122(Unk0x1aae122),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VfxEmitterDefinitionDataSpawnShape {
    Unk0xee39916f(Unk0xee39916f),
    VfxShapeSphere(VfxShapeSphere),
    VfxShapeCylinder(VfxShapeCylinder),
    VfxShapeBox(VfxShapeBox),
    VfxShapeLegacy(VfxShapeLegacy),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VfxEmitterDefinitionDataPrimitive {
    VfxPrimitiveCameraTrail(VfxPrimitiveCameraTrail),
    VfxPrimitiveCameraSegmentBeam(VfxPrimitiveCameraSegmentBeam),
    VfxPrimitiveAttachedMesh(VfxPrimitiveAttachedMesh),
    VfxPrimitiveRay,
    VfxPrimitivePlanarProjection(VfxPrimitivePlanarProjection),
    VfxPrimitiveArbitraryQuad,
    VfxPrimitiveCameraUnitQuad,
    VfxPrimitiveBeam(VfxPrimitiveBeam),
    VfxPrimitiveMesh(VfxPrimitiveMesh),
    Unk0x8df5fcf7,
    VfxPrimitiveArbitraryTrail(VfxPrimitiveArbitraryTrail),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConditionBoolClipDataUpdater {
    IsRangedParametricUpdater,
    LogicDriverFloatParametricUpdater(LogicDriverFloatParametricUpdater),
    IsInTerrainParametricUpdater,
    IsHomeguardParametricUpdater,
    LookAtSpellTargetHeightOffsetParametricUpdater,
    TurnAngleParametricUpdater,
    LookAtInterestAngleParametricUpdater,
    IsAllyParametricUpdater,
    LogicDriverBoolParametricUpdater(LogicDriverBoolParametricUpdater),
    EquippedGearParametricUpdater,
    LookAtSpellTargetAngleParametricUpdater,
    DisplacementParametricUpdater,
    SlopeAngleParametricUpdater,
    LookAtGoldRedirectTargetAngleParametricUpdater,
    Unk0xe7b61183(Unk0xe7b61183),
    FacingAndMovementAngleParametricUpdater,
    AttackSpeedParametricUpdater,
    LookAtInterestDistanceParametricUpdater,
    TurnAngleRemainingParametricUpdater,
    SkinScaleParametricUpdater,
    IsTurningParametricUpdater,
    TotalTurnAngleParametricUpdater,
    MoveSpeedParametricUpdater,
    IsMovingParametricUpdater,
    FacingParametricUpdater,
    ParBarPercentParametricUpdater,
    MovementDirectionParametricUpdater,
    LookAtSpellTargetDistanceParametricUpdater,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TargeterDefinitionRangeMFadeBehavior {
    FadeOverTimeBehavior(FadeOverTimeBehavior),
    FadeToExplicitValueBehavior(FadeToExplicitValueBehavior),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Unk0xc76c1b9aModifiers {
    Unk0x51445de9(Unk0x51445de9),
    Unk0x557bb273(Unk0x557bb273),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SkinCharacterDataPropertiesUnknownEnumField {
    Unk0xe7ee4f28(Unk0xe7ee4f28),
    Unk0xc96d9140(Unk0xc96d9140),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SkinMeshDataPropertiesRigPoseModifierData {
    ConformToPathRigPoseModifierData(ConformToPathRigPoseModifierData),
    Unk0xe6147387(Unk0xe6147387),
    SpringPhysicsRigPoseModifierData(SpringPhysicsRigPoseModifierData),
    SyncedAnimationRigPoseModifierData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BarracksMinionConfigWaveBehavior {
    ConstantWaveBehavior(ConstantWaveBehavior),
    RotatingWaveBehavior(RotatingWaveBehavior),
    InhibitorWaveBehavior(InhibitorWaveBehavior),
    TimedVariableWaveBehavior(TimedVariableWaveBehavior),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum JunglePathRecommendationOrderJunglePath {
    TerminatePath,
    TakeCamp(TakeCamp),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VfxMaterialDefinitionDataMaterialDrivers {
    VfxFloatOverLifeMaterialDriver(VfxFloatOverLifeMaterialDriver),
    VfxColorOverLifeMaterialDriver(VfxColorOverLifeMaterialDriver),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ContextualRuleMConditions {
    Unk0xd55b5c23,
    ContextualConditionSpellLevel(ContextualConditionSpellLevel),
    ContextualConditionSpell(ContextualConditionSpell),
    ContextualConditionBuffCounterSetToZeroAfterLimitReached(
        ContextualConditionBuffCounterSetToZeroAfterLimitReached,
    ),
    ContextualConditionObjectiveTakeByMyTeam(ContextualConditionObjectiveTakeByMyTeam),
    ContextualConditionSpellBuffName(ContextualConditionSpellBuffName),
    ContextualConditionCharacter(ContextualConditionCharacter),
    ContextualConditionItemId(ContextualConditionItemId),
    ContextualConditionKillCount(ContextualConditionKillCount),
    Unk0x2363fb10(Unk0x2363fb10),
    ContextualConditionTeammateDeathsNearby(ContextualConditionTeammateDeathsNearby),
    ContextualConditionMoveDistance(ContextualConditionMoveDistance),
    Unk0x61b427f(Unk0x61b427f),
    ContextualConditionWinningTeam(ContextualConditionWinningTeam),
    ContextualConditionMapId(ContextualConditionMapId),
    ContextualConditionSpellSlot(ContextualConditionSpellSlot),
    ContextualConditionNumberOfCharactersNearTargetPos(
        ContextualConditionNumberOfCharactersNearTargetPos,
    ),
    ContextualConditionNegation(ContextualConditionNegation),
    Unk0x4ab36eb5(Unk0x4ab36eb5),
    ContextualConditionMarkerName(ContextualConditionMarkerName),
    ContextualConditionChanceToPlay(ContextualConditionChanceToPlay),
    ContextualConditionBuffCounterChanged(ContextualConditionBuffCounterChanged),
    ContextualConditionMultikillSize(ContextualConditionMultikillSize),
    ContextualConditionAnyOtherHero(ContextualConditionAnyOtherHero),
    ContextualConditionSpellName(ContextualConditionSpellName),
    ContextualConditionBuffCounterReachedLimitFromZero(
        ContextualConditionBuffCounterReachedLimitFromZero,
    ),
    ContextualConditionItemVoGroup(ContextualConditionItemVoGroup),
    ContextualConditionNeutralCampId(ContextualConditionNeutralCampId),
    ContextualConditionRuleCooldown(ContextualConditionRuleCooldown),
    ContextualConditionMapRegionName(ContextualConditionMapRegionName),
    ContextualConditionGameTimer(ContextualConditionGameTimer),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x47f13ab0 {
    pub unk_0xcf19cb5d: Unk0x770f7888,
    pub unk_0xe4f7105d: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolSpellDesc {
    pub desc: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedDistanceIgnoringTerrain {
    pub m_minimum_gap_between_terrain_walls: Option<f32>,
    pub m_maximum_distance: f32,
    pub scan_width_override: Option<f32>,
    pub m_maximum_terrain_walls_to_skip: Option<u32>,
    pub m_targeter_definition: TargeterDefinitionSkipTerrain,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionChanceToPlay {
    pub m_percent_chance_to_play: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConditionFloatPairData {
    pub m_hold_animation_to_lower: Option<f32>,
    pub m_clip_name: u32,
    pub m_value: Option<f32>,
    pub m_hold_animation_to_higher: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KeyFrameFloatClipReaderDriver {
    pub clip_accessory_to_read: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x8ad25772 {
    pub system: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellModifier {
    pub description_append_priority: Option<u32>,
    pub description_append_tra: Option<String>,
    pub m_modifier_id: u32,
    pub m_calculation_stat_conversions: Option<Vec<RatioConversion>>,
    pub m_spell_does_not_include_stat_scaling: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatStoneEventToTrack {
    pub stat_filters: Option<Vec<StatStoneEventToTrackStatFilters>>,
    pub event_to_track: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolEducationData {
    pub first_item: i32,
    pub skill_order: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x557bb273 {
    pub value: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ObjectTags {
    pub m_object_tag_list: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TypePath, Asset)]
#[serde(rename_all = "camelCase")]
pub struct SpellObject {
    pub m_buff: Option<BuffData>,
    pub object_name: String,
    pub m_script_name: String,
    pub m_spell: Option<SpellDataResource>,
    pub bot_data: Option<BotsSpellData>,
    pub cc_behavior_data: Option<CcBehaviorData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdaterResourceData {
    pub m_updater_data_list: Option<Vec<UpdaterData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SelectorPairData {
    pub m_clip_name: u32,
    pub m_probability: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataValueVector {
    pub spell_data_values: Option<Vec<SpellDataValue>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedFloatVariableData {
    pub times: Option<Vec<f32>>,
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub values: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xcf4a55da {
    pub overlays: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionCharacter {
    pub m_child_conditions: Option<Vec<ContextualConditionCharacterMChildConditions>>,
    pub m_character_type: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualActionTriggerEvent {
    pub m_hashed_situation_trigger: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEventData {
    pub m_name: Option<u32>,
    pub skip_if_past_end_frame: Option<bool>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub scale: Option<f32>,
    pub m_is_self_only: Option<bool>,
    pub m_is_kill_event: Option<bool>,
    pub m_is_loop: Option<bool>,
    pub m_end_frame: Option<f32>,
    pub m_effect_name: Option<String>,
    pub m_effect_key: Option<u32>,
    pub m_particle_event_data_pair_list: Option<Vec<ParticleEventDataPair>>,
    pub m_start_frame: Option<f32>,
    pub m_enemy_effect_key: Option<u32>,
    pub m_is_detachable: Option<bool>,
    pub m_scale_play_speed_with_animation: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CensoredImage {
    pub image: String,
    pub uncensored_images: Option<HashMap<u32, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xfde6a2d7 {
    pub team: Option<u32>,
    pub unk_0xdbde2288: Vec<Unk0x82cab1b3>,
    pub barracks_config: u32,
    pub unk_0xdb6ea1a7: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellRankUpRequirements {
    pub m_requirements: Option<Vec<SpellRankUpRequirementsMRequirements>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PhysicsMovement {
    pub m_wall_sliding_friction_multiplier: f32,
    pub m_wall_sliding: bool,
    pub m_target_height_augment: Option<f32>,
    pub m_offset_initial_target_height: Option<f32>,
    pub m_tracks_target: bool,
    pub m_start_bone_name: String,
    pub m_drag: f32,
    pub m_initial_speed: f32,
    pub m_lifetime: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapNavGrid {
    pub nav_grid_path: String,
    pub nav_grid_config: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClearTargetAndKeepMoving {
    pub m_override_movement: Option<FixedSpeedMovement>,
    pub m_override_height_augment: Option<f32>,
    pub m_override_range: Option<f32>,
    pub let_server_drive_target_position: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtomicClipData {
    pub m_event_data_map: Option<HashMap<u32, AtomicClipDataMEventDataMap>>,
    pub m_sync_group_data_name: Option<u32>,
    pub m_tick_duration: Option<f32>,
    pub end_frame: Option<f32>,
    pub start_frame: Option<f32>,
    pub m_updater_resource_data: Option<UpdaterResourceData>,
    pub accessorylist: Option<Vec<KeyFrameFloatMapClipAccessoryData>>,
    pub m_track_data_name: u32,
    pub m_mask_data_name: Option<u32>,
    pub m_animation_resource_data: AnimationResourceData,
    pub m_flags: Option<u32>,
    pub m_animation_interruption_group_names: Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FloatPerSpellLevel {
    pub m_per_level_values: Option<Vec<f32>>,
    pub m_value_type: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionMapId {
    pub m_map_i_ds: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x82cab1b3 {
    pub position: Option<u16>,
    pub lane: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WallDetection {
    pub detection_range: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncedAnimationEventData {
    pub m_lerp_time: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterLevelRequirement {
    pub m_level: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapGroup {
    pub transform: Mat4,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionNumberOfCharactersNearTargetPos {
    pub m_team_compare_op: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x5b2fdd66 {
    pub value: AnimationFractionDynamicMaterialFloatDriver,
    pub add: FloatLiteralMaterialDriver,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellRankIntDriver {
    pub spell_slot: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Defaultvisibility {
    pub trail_time_to_consider_for_visibility: Option<f32>,
    pub m_perception_bubble_radius: Option<f32>,
    pub m_visible_to_owner_team_only: Option<bool>,
    pub m_target_controls_visibility: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasAtleastNSubRequirementsCastRequirement {
    pub m_sub_requirements: Vec<HasBuffCastRequirement>,
    pub m_successes_required: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellEffectAmount {
    pub value: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionBuffCounterReachedLimitFromZero {
    pub m_counter_highest_reached: u8,
    pub compare_op: Option<u8>,
    pub m_buff: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolAlternateForm {
    pub name: String,
    pub spells: Option<Vec<String>>,
    pub champion: Option<String>,
    pub the_switch: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct VfxSystemDefinitionData {
    pub override_scale_cap: Option<f32>,
    pub asset_remapping_table: Option<Vec<VfxAssetRemap>>,
    pub voice_over_on_create_default: Option<String>,
    pub voice_over_persistent_default: Option<String>,
    pub m_eye_candy: Option<bool>,
    pub drawing_layer: Option<u8>,
    pub simple_emitter_definition_data: Option<Vec<VfxEmitterDefinitionData>>,
    pub transform: Option<Mat4>,
    pub self_illumination: Option<f32>,
    pub audio_parameter_time_scaled_duration: Option<f32>,
    pub sound_persistent_default: Option<String>,
    pub unk_0xf97b1289: Option<Unk0x7fb92f53>,
    pub hud_anchor_position_from_world_projection: Option<bool>,
    pub sound_on_create_default: Option<String>,
    pub build_up_time: Option<f32>,
    pub m_is_pose_afterimage: Option<bool>,
    pub material_override_definitions: Option<Vec<VfxMaterialOverrideDefinitionData>>,
    pub unk_0x9836cd87: Option<u8>,
    pub complex_emitter_definition_data: Option<Vec<VfxEmitterDefinitionData>>,
    pub particle_name: String,
    pub audio_parameter_flex_id: Option<i32>,
    pub hud_layer_dimension: Option<f32>,
    pub flags: Option<u16>,
    pub visibility_radius: Option<f32>,
    pub clock_to_use: Option<u8>,
    pub particle_path: String,
    pub scale_dynamically_with_attached_bone: Option<bool>,
    pub unk_0x8b301739: Option<Unk0x75e34c40>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlendingSwitchMaterialDriver {
    pub m_default_value: Box<BlendingSwitchMaterialDriverMDefaultValue>,
    pub m_blend_time: Option<f32>,
    pub m_override_blend_times: Option<Vec<f32>>,
    pub m_elements: Box<Vec<SwitchMaterialDriverElement>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xeb997689 {
    pub unk_0x9e5c6a2c: Option<u32>,
    pub transform: Mat4,
    pub definition: Unk0xfcb92181,
    pub name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IsAnimationPlayingDynamicMaterialBoolDriver {
    pub m_animation_names: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MutatorMapVisibilityController {
    pub path_hash: u32,
    pub mutator_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DistanceToPlayerMaterialFloatDriver {
    pub min_distance: f32,
    pub max_distance: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionIsSelf {
    pub is_self: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetingParameters {
    pub unk_0x791c5fa3: Option<bool>,
    pub m_spell_flags: Option<u32>,
    pub targeting_param_name: Option<String>,
    pub unk_0x9845aa67: Option<bool>,
    pub m_affects_type_flags: u32,
    pub range_value: TargetingParametersRangeValue,
    pub m_affects_status_flags: Option<u32>,
    pub unit_object_tags: Option<ObjectTags>,
    pub unk_0xfc462d60: Option<Vec<Unk0xe90af953>>,
    pub exit_conditions: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConstantWaveBehavior {
    pub spawn_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xc7e628b9 {
    pub unk_0xa2877ddb: u32,
    pub unk_0xd00e123a: u32,
    pub unk_0xe5bc4229: u32,
    pub spell: u32,
    pub unk_0x877e4953: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xd5c9eb1 {
    pub event_name: u32,
    pub unk_0x1004c9c8: HashMap<u32, Unk0x56bb851>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatBySubPartCalculationPart {
    pub m_stat_formula: Option<u8>,
    pub unk_0xa8cb9c14: Option<bool>,
    pub m_stat: Option<u8>,
    pub m_subpart: Box<StatBySubPartCalculationPartMSubpart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FloatLiteralMaterialDriver {
    pub m_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AutoSpellCastInfo {
    pub auto_spell_cast: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x48f3fe52 {
    pub unk_0xb9cb9ce8: Vec<Unk0x7084628f>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapAudio {
    pub transform: Mat4,
    pub name: String,
    pub event_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SelectorClipData {
    pub m_selector_pair_data_list: Vec<SelectorPairData>,
    pub m_flags: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldAccelerationDefinitionData {
    pub acceleration: Option<ValueVector3>,
    pub is_local_space: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeCylinder {
    pub radius: Option<f32>,
    pub flags: Option<u8>,
    pub height: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x429a2180 {
    pub unk_0x1f2e5fd0: Unk0x1f2e5fd0,
    pub scoreboard_timer: Option<u16>,
    pub camp_level: Option<u16>,
    pub team: u32,
    pub minimap_icon_offset: Option<Vec3>,
    pub stop_spawn_time_secs: Option<f32>,
    pub tags: Option<Vec<u32>>,
    pub unk_0x7d27af7f: Option<bool>,
    pub minimap_icon: Option<String>,
    pub unk_0x5a4ef4e7: u32,
    pub reveal_event: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlexValueVector3 {
    pub m_value: Option<ValueVector3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasBuffOfTypeBoolDriver {
    pub buff_type: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionCharacterUnitTags {
    pub m_unit_tags: ObjectTags,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x3c995caf {
    pub segments: Vec<Vec3>,
    pub name: String,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetingPriorityList {
    pub m_spell_flags: Option<u32>,
    pub targeting_parameters_list: Vec<TargetingParameters>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TriggerOnDelay {
    pub m_actions: Vec<TriggerOnDelayMActions>,
    pub m_delay: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterAudio {
    pub sound_persistent: Option<String>,
    pub sound_on_create: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualActionCooldownModifications {
    pub dont_reset_timer: Option<bool>,
    pub ignore_timer: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ColorGraphMaterialDriver {
    pub colors: VfxAnimatedColorVariableData,
    pub driver: Box<BlendingSwitchMaterialDriverMDefaultValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RotatingWaveBehavior {
    pub spawn_counts_by_wave: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResimulateTrailVfxOnEnterVisibility {
    pub simulation_frames: u32,
    pub cycles: u32,
    pub time_to_resimulate: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinEmblem {
    pub m_loading_screen_anchor: Option<u32>,
    pub m_emblem_data: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LerpMaterialDriver {
    pub m_turn_on_time_sec: Option<f32>,
    pub m_on_value: Option<f32>,
    pub m_turn_off_time_sec: Option<f32>,
    pub m_use_broken_old_interpolation: Option<bool>,
    pub m_bool_driver: Box<BlendingSwitchMaterialDriverMDefaultValue>,
    pub m_off_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xee39916f {
    pub emit_offset: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedColorVariableData {
    pub values: Option<Vec<Vec4>>,
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub times: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionSpellSlot {
    pub m_spell_slot: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemapVec4MaterialDriver {
    pub output_max_value: Vec4,
    pub driver: AbilityResourceDynamicMaterialFloatDriver,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxBeamDefinitionData {
    pub m_birth_tiling_size: Option<ValueVector3>,
    pub m_mode: Option<u8>,
    pub m_segments: Option<i32>,
    pub m_is_color_binded_with_distance: Option<bool>,
    pub m_local_space_source_offset: Option<Vec3>,
    pub m_animated_color_with_distance: Option<ValueColor>,
    pub m_trail_mode: Option<u8>,
    pub m_local_space_target_offset: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialShaderParamDef {
    pub name: String,
    pub value: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionCharacterHealth {
    pub m_compare_op: u8,
    pub m_target_health: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FollowTerrainHeightSolver {
    pub m_max_slope: Option<f32>,
    pub m_height_offset: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomTargeterDefinitions {
    pub m_targeter_definitions: Vec<CustomTargeterDefinitionsMTargeterDefinitions>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x20194a16 {
    pub unk_0xfcea40c4: bool,
    pub resource_resolver: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrackMouseMovement {
    pub m_anti_lag_delay: f32,
    pub m_turn_radius_by_level: Vec<f32>,
    pub m_project_target_to_cast_range: Option<bool>,
    pub m_tracks_target: bool,
    pub m_min_speed: f32,
    pub m_target_bone_name: String,
    pub m_initial_speed: f32,
    pub m_start_bone_name: String,
    pub m_max_speed: f32,
    pub m_acceleration: f32,
    pub m_use_ground_height_at_target: bool,
    pub m_use_height_offset_at_end: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionAoe {
    pub dynamic_game_calc_size_scalar: Option<GameCalculationModified>,
    pub texture_radius_override_name: Option<String>,
    pub constraint_pos_locator: Option<DrawablePositionLocator>,
    pub max_range_size_scalar: Option<TargeterDefinitionAoeScalar>,
    pub is_constrained_to_range: Option<bool>,
    pub texture_orientation: Option<u32>,
    pub center_locator: Option<DrawablePositionLocator>,
    pub constraint_range: Option<FloatPerSpellLevel>,
    pub override_radius: Option<FloatPerSpellLevel>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemRecOverrideRange {
    pub items: Vec<u32>,
    pub max_completed_items: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x64ee2fb1 {
    pub id: u32,
    pub max_scale: f32,
    pub color: Unk0x634a3a64,
    pub texture: Unk0x1003c990,
    pub min_scale: f32,
    pub size: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SameTeamCastRequirement {
    pub m_invert_result: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValueVector3 {
    pub dynamics: Option<VfxAnimatedVector3fVariableData>,
    pub constant_value: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameCalculationModified {
    pub m_modified_game_calculation: u32,
    pub tooltip_only: Option<bool>,
    pub m_multiplier: StatBySubPartCalculationPartMSubpart,
    pub m_expanded_tooltip_calculation_display: Option<u8>,
    pub m_override_spell_level: Option<i32>,
    pub m_simple_tooltip_calculation_display: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x19da44b2 {
    pub unk_0x44146c9d: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasBuffDynamicMaterialBoolDriver {
    pub m_script_name: Option<String>,
    pub unk_0x149271dd: Option<bool>,
    pub spell: Option<u32>,
    pub m_deactivate_early_seconds: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldDragDefinitionData {
    pub radius: Option<ValueFloat>,
    pub strength: Option<ValueFloat>,
    pub position: Option<ValueVector3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransitionClipBlendData {
    pub m_clip_name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xd91a223 {
    pub unk_0xe2e5b6dd: Vec<u32>,
    pub unk_0x68309c0b: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemRecommendationOverrideContext {
    pub m_mode_name_string_id: u32,
    pub m_map_id: Option<u32>,
    pub m_position: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LinearTransformProcessorData {
    pub m_multiplier: f32,
    pub m_increment: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct AnimationGraphData {
    pub m_sync_group_data_map: Option<HashMap<u32, SyncGroupData>>,
    pub m_blend_data_table: Option<HashMap<u64, AnimationGraphDataMBlendDataTable>>,
    pub m_track_data_map: HashMap<u32, TrackData>,
    pub m_clip_data_map: Option<HashMap<u32, AnimationGraphDataMClipDataMap>>,
    pub m_cascade_blend_value: Option<f32>,
    pub m_use_cascade_blend: Option<bool>,
    pub m_mask_data_map: Option<HashMap<u32, MaskData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x7fb92f53 {
    pub unk_0x28de30d6: Option<f32>,
    pub unk_0x3c475337: f32,
    pub unk_0xc865acd9: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialDef {
    pub child_techniques: Option<Vec<StaticMaterialChildTechniqueDef>>,
    pub param_values: Vec<StaticMaterialShaderParamDef>,
    pub dynamic_material: Option<DynamicMaterialDef>,
    pub r#type: Option<u32>,
    pub sampler_values: Option<Vec<StaticMaterialShaderSamplerDef>>,
    pub switches: Option<Vec<StaticMaterialSwitchDef>>,
    pub shader_macros: Option<HashMap<String, String>>,
    pub techniques: Vec<StaticMaterialTechniqueDef>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdleParticlesVisibilityEventData {
    pub m_start_frame: Option<f32>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub m_show: Option<bool>,
    pub m_is_self_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x125a3586 {
    pub unk_0xe61bf09e: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xe1555e0a {
    pub unk_0x9a676645: Option<u32>,
    pub augment_group: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedVector3fVariableData {
    pub times: Option<Vec<f32>>,
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub values: Option<Vec<Vec3>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxProbabilityTableData {
    pub key_values: Option<Vec<f32>>,
    pub key_times: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialSwitchDef {
    pub group: Option<String>,
    pub on: Option<bool>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogicDriverBoolParametricUpdater {
    pub driver: Option<BlendingSwitchMaterialDriverMDefaultValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x7faa90a0 {
    pub play_idle_animation: Option<bool>,
    pub team: u32,
    pub skin: String,
    pub character_record: String,
    pub idle_animation_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxChildParticleSetDefinitionData {
    pub children_identifiers: Option<Vec<VfxChildIdentifier>>,
    pub bone_to_spawn_at: Option<Vec<String>>,
    pub child_emit_on_death: Option<bool>,
    pub children_probability: Option<ValueFloat>,
    pub parent_inheritance_definition: Option<VfxParentInheritanceParams>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataResourceClient {
    pub m_tooltip_data: Option<TooltipInstanceSpell>,
    pub m_left_click_spell_action: Option<u32>,
    pub m_missile_targeter_definitions: Option<Vec<MissileAttachedTargetingDefinition>>,
    pub m_use_death_recap_tooltip_from_another_spell: Option<u32>,
    pub m_custom_targeter_definitions: Option<HashMap<u32, CustomTargeterDefinitions>>,
    pub m_spawning_ui_definition: Option<SpawningUiDefinition>,
    pub m_right_click_spell_action: Option<u32>,
    pub m_targeter_definitions: Option<Vec<CustomTargeterDefinitionsMTargeterDefinitions>>,
    pub m_use_tooltip_from_another_spell: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedTimeMovement {
    pub m_travel_time: f32,
    pub m_start_bone_name: Option<String>,
    pub m_use_height_offset_at_end: Option<bool>,
    pub m_target_bone_name: Option<String>,
    pub m_tracks_target: Option<bool>,
    pub m_project_target_to_cast_range: Option<bool>,
    pub m_start_bone_skin_overrides: Option<HashMap<u32, String>>,
    pub m_use_ground_height_at_target: Option<bool>,
    pub m_target_height_augment: Option<f32>,
    pub m_offset_initial_target_height: Option<f32>,
    pub m_infer_direction_from_facing_if_needed: Option<bool>,
    pub m_visuals_track_hidden_targets: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Float4LiteralMaterialDriver {
    pub value: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemRecommendationContext {
    pub m_is_default_position: Option<bool>,
    pub m_starting_item_bundles: Vec<ItemRecommendationItemList>,
    pub m_popular_items: Vec<u32>,
    pub m_starting_item_matrix: ItemRecommendationMatrix,
    pub m_champion_id: u32,
    pub m_mode_name_string_id: u32,
    pub m_completed_item_matrix: ItemRecommendationMatrix,
    pub m_map_id: u32,
    pub m_position: u32,
    pub upgrade_choices: Option<HashMap<u32, ItemRecommendationItemList>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameCalculationConditional {
    pub m_conditional_calculation_requirements: HasBuffCastRequirement,
    pub m_default_game_calculation: u32,
    pub m_conditional_game_calculation: u32,
    pub tooltip_only: Option<bool>,
    pub m_expanded_tooltip_calculation_display: Option<u8>,
    pub m_simple_tooltip_calculation_display: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValueVector2 {
    pub constant_value: Option<Vec2>,
    pub dynamics: Option<VfxAnimatedVector2fVariableData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x6fb748e3 {
    pub files: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetHasUnitTagFilter {
    pub unit_tags: ObjectTags,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x2363fb10 {
    pub animation_name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolInheritsData {
    pub recommended: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NamedDataValueCalculationPart {
    pub m_data_value: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionMultiAoe {
    pub right_texture_name: Option<String>,
    pub left_texture_name: String,
    pub override_min_cast_range: FloatPerSpellLevel,
    pub center_locator: DrawablePositionLocator,
    pub override_aoe_radius: Option<FloatPerSpellLevel>,
    pub override_max_cast_range: FloatPerSpellLevel,
    pub num_of_inner_aoe: Option<u32>,
    pub inner_texture_name: String,
    pub angel_offset_radian: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CircleMovement {
    pub m_lifetime: f32,
    pub m_start_bone_name: String,
    pub m_linear_velocity: Option<f32>,
    pub m_angular_velocity: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionItemVoGroup {
    pub m_item_vo_group_hash: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionCharacterHasTimeRemainingForAnimation {
    pub m_min_remaining_time: f32,
    pub m_animation_name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeMissileWidth {
    pub width_value: f32,
    pub width_change_type: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConformToPathEventData {
    pub m_is_self_only: Option<bool>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub m_mask_data_name: u32,
    pub m_end_frame: Option<f32>,
    pub m_blend_out_time: Option<f32>,
    pub m_start_frame: Option<f32>,
    pub m_name: Option<u32>,
    pub m_blend_in_time: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataValue {
    pub m_values: Option<Vec<f32>>,
    pub m_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x11a27f9e {
    pub visibility: Option<u32>,
    pub styles: Vec<Unk0x3c52b66>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncCircleMovement {
    pub m_visuals_track_hidden_targets: Option<bool>,
    pub m_angular_velocity: f32,
    pub m_start_bone_name: Option<String>,
    pub m_axis_of_rotation: Option<u8>,
    pub m_offset_initial_target_height: Option<f32>,
    pub m_lifetime: Option<f32>,
    pub m_rotate_around_caster_facing_direction: Option<bool>,
    pub m_target_bone_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xfb16e4be {
    pub order_types: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemRecommendationMatrix {
    pub mrows: Vec<ItemRecommendationMatrixRow>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldAttractionDefinitionData {
    pub radius: Option<ValueFloat>,
    pub acceleration: Option<ValueFloat>,
    pub position: Option<ValueVector3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellLockDeltaTimeData {
    pub m_spell_lock_delta_time_calculation: GameCalculation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataResource {
    pub m_caster_position_end_of_cast_update: Option<u8>,
    pub m_hit_effect_name: Option<String>,
    pub m_animation_name: Option<String>,
    pub m_pre_cast_lockout_delta_time_data: Option<SpellLockDeltaTimeData>,
    pub m_hit_effect_player_key: Option<u32>,
    pub m_orient_radius_texture_from_player: Option<bool>,
    pub unk_0xb08bc498: Option<HashMap<u32, SpellEffectAmount>>,
    pub m_cast_time: Option<f32>,
    pub m_project_target_to_cast_range: Option<bool>,
    pub m_cursor_changes_in_terrain: Option<bool>,
    pub lua_on_missile_update_distance_interval: Option<f32>,
    pub m_hit_effect_key: Option<u32>,
    pub m_resource_resolvers: Option<Vec<u32>>,
    pub cooldown_time: Option<Vec<f32>>,
    pub m_spell_cooldown_or_sealed_queue_threshold: Option<f32>,
    pub m_img_icon_name: Option<Vec<String>>,
    pub m_coefficient: Option<f32>,
    pub m_cant_cancel_while_winding_up: Option<bool>,
    pub m_character_passive_buffs: Option<Vec<CharacterPassiveData>>,
    pub data_values_mode_override: Option<HashMap<u32, SpellDataValueVector>>,
    pub m_apply_attack_effect: Option<bool>,
    pub m_locked_spell_origination_cast_id: Option<bool>,
    pub m_coefficient2: Option<f32>,
    pub m_ignore_anim_continue_until_cast_frame: Option<bool>,
    pub b_have_hit_bone: Option<bool>,
    pub m_cooldown_not_affected_by_cdr: Option<bool>,
    pub m_use_autoattack_cast_time_data: Option<UseAutoattackCastTimeData>,
    pub m_update_rotation_when_casting: Option<bool>,
    pub delay_total_time_percent: Option<f32>,
    pub cast_range_display_override: Option<Vec<f32>>,
    pub m_can_trigger_charge_spell_while_disabled: Option<bool>,
    pub m_doesnt_break_channels: Option<bool>,
    pub m_missile_effect_enemy_key: Option<u32>,
    pub img_icon_path: Option<String>,
    pub m_post_cast_lockout_delta_time_data: Option<SpellLockDeltaTimeData>,
    pub m_cast_range_growth_duration: Option<Vec<f32>>,
    pub m_cast_range_growth_max: Option<Vec<f32>>,
    pub m_affects_type_flags: Option<u32>,
    pub mana_ui_override: Option<Vec<f32>>,
    pub m_platform_spell_info: Option<PlatformSpellInfo>,
    pub m_hit_effect_player_name: Option<String>,
    pub m_missile_effect_name: Option<String>,
    pub can_cast_while_disabled: Option<bool>,
    pub m_client_data: Option<SpellDataResourceClient>,
    pub m_float_vars_decimals: Option<Vec<i32>>,
    pub m_cant_cancel_while_winding_up_targeting_champion: Option<bool>,
    pub m_channel_duration: Option<Vec<f32>>,
    pub m_can_move_while_channeling: Option<bool>,
    pub m_pingable_while_disabled: Option<bool>,
    pub m_start_cooldown: Option<f32>,
    pub spell_event_to_audio_event_suffix: Option<HashMap<u32, String>>,
    pub m_minimap_icon_rotation: Option<bool>,
    pub cast_frame: Option<f32>,
    pub m_turn_speed_scalar: Option<f32>,
    pub m_ammo_count_hidden_in_ui: Option<bool>,
    pub m_is_delayed_by_cast_locked: Option<bool>,
    pub m_ammo_not_affected_by_cdr: Option<bool>,
    pub m_casting_breaks_stealth: Option<bool>,
    pub mana: Option<Vec<f32>>,
    pub m_animation_winddown_name: Option<String>,
    pub cast_range_use_bounding_boxes: Option<bool>,
    pub m_ammo_used: Option<Vec<i32>>,
    pub m_ai_data: Option<AiSpellData>,
    pub always_snap_facing: Option<bool>,
    pub m_animation_loop_name: Option<String>,
    pub m_particle_start_offset: Option<Vec3>,
    pub m_minimap_icon_name: Option<String>,
    pub unk_0x288b8edc: Option<SpellDataResourceUnknownEnumField>,
    pub missile_effect_maximum_angle_degrees: Option<f32>,
    pub m_max_ammo: Option<Vec<i32>>,
    pub m_roll_for_critical_hit: Option<bool>,
    pub b_is_toggle_spell: Option<bool>,
    pub m_keyword_when_acquired: Option<String>,
    pub m_show_channel_bar: Option<bool>,
    pub m_cast_range_growth_start_time: Option<Vec<f32>>,
    pub unk_0x48201b0d: Option<f32>,
    pub m_belongs_to_avatar: Option<bool>,
    pub cast_cone_distance: Option<f32>,
    pub selection_priority: Option<u32>,
    pub cast_target_additional_units_radius: Option<f32>,
    pub m_cancel_charge_on_recast_time: Option<f32>,
    pub m_cursor_changes_in_grass: Option<bool>,
    pub m_targeting_type_data: Option<SpellDataResourceMTargetingTypeData>,
    pub m_channel_is_interrupted_by_attacking: Option<bool>,
    pub m_override_attack_time: Option<OverrideAttackTimeData>,
    pub m_dimension_behavior: Option<u8>,
    pub m_line_drag_length: Option<f32>,
    pub missile_speed: Option<f32>,
    pub cast_radius: Option<Vec<f32>>,
    pub m_ammo_recharge_time: Option<Vec<f32>>,
    pub m_missile_effect_key: Option<u32>,
    pub m_excluded_unit_tags: Option<ObjectTags>,
    pub m_cast_requirements_caster: Option<Vec<SpellDataResourceMCastRequirementsCaster>>,
    pub cant_cast_while_rooted: Option<bool>,
    pub cannot_be_suppressed: Option<bool>,
    pub m_charge_update_interval: Option<f32>,
    pub use_animator_framerate: Option<bool>,
    pub m_apply_attack_damage: Option<bool>,
    pub m_disable_cast_bar: Option<bool>,
    pub m_casting_reveals_caster_stealth: Option<bool>,
    pub m_hit_effect_orient_type: Option<u32>,
    pub m_considered_as_auto_attack: Option<bool>,
    pub m_use_charge_channeling: Option<bool>,
    pub m_hide_range_indicator_when_casting: Option<bool>,
    pub m_after_effect_key: Option<u32>,
    pub show_channel_bar_per_spell_level_override: Option<Vec<bool>>,
    pub m_casting_breaks_stealth_while_attached: Option<bool>,
    pub m_missile_effect_player_key: Option<u32>,
    pub m_cast_requirements_target: Option<Vec<SpellDataResourceMCastRequirementsCaster>>,
    pub can_only_cast_while_dead: Option<bool>,
    pub should_receive_input_events: Option<bool>,
    pub data_values: Option<Vec<SpellDataValue>>,
    pub flags: Option<u32>,
    pub passive_spell_affected_by_cooldown: Option<bool>,
    pub m_no_winddown_if_cancelled: Option<bool>,
    pub m_look_at_policy: Option<u32>,
    pub m_missile_spec: Option<MissileSpecification>,
    pub m_effect_amount: Option<Vec<SpellEffectAmount>>,
    pub m_spell_calculations: Option<HashMap<u32, SpellDataResourceMSpellCalculations>>,
    pub m_use_minimap_targeting: Option<bool>,
    pub delay_cast_offset_percent: Option<f32>,
    pub m_apply_material_on_hit_sound: Option<bool>,
    pub m_does_not_consume_mana: Option<bool>,
    pub cast_cone_angle: Option<f32>,
    pub m_minimap_icon_display_flag: Option<u16>,
    pub unk_0xf9c2333e: Option<HashMap<u32, SpellEffectAmount>>,
    pub m_is_disabled_while_dead: Option<bool>,
    pub m_cost_always_shown_in_ui: Option<bool>,
    pub cast_radius_secondary: Option<Vec<f32>>,
    pub can_cast_or_queue_while_casting: Option<bool>,
    pub cast_range: Option<Vec<f32>>,
    pub m_missile_effect_player_name: Option<String>,
    pub m_spell_reveals_champion: Option<bool>,
    pub m_required_unit_tags: Option<ObjectTags>,
    pub m_hit_bone_name: Option<String>,
    pub b_have_hit_effect: Option<bool>,
    pub m_affects_status_flags: Option<u32>,
    pub missile_effect_max_turn_speed_degrees_per_second: Option<f32>,
    pub m_after_effect_name: Option<String>,
    pub unk_0x66769fb4: Option<bool>,
    pub m_does_not_consume_cooldown: Option<bool>,
    pub auto_spell_cast_info: Option<AutoSpellCastInfo>,
    pub m_pre_cast_lockout_delta_time: Option<f32>,
    pub m_cant_cancel_while_channeling: Option<bool>,
    pub unk_0xf4ca428f: Option<u8>,
    pub m_cast_type: Option<u32>,
    pub m_post_cast_lockout_delta_time: Option<f32>,
    pub m_do_not_need_to_face_target: Option<bool>,
    pub m_ignore_range_check: Option<bool>,
    pub m_alternate_name: Option<String>,
    pub m_spell_tags: Option<Vec<String>>,
    pub unk_0x8958fee2: Option<Unk0x8958fee2>,
    pub targeting_forgiveness_definitions: Option<Vec<TargetingForgivenessDefinitions>>,
    pub m_line_width: Option<f32>,
    pub m_channel_is_interrupted_by_disables: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionRuleCooldown {
    pub m_rule_cooldown: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueFloat {
    pub constant_value: Option<f32>,
    pub dynamics: VfxAnimatedFloatVariableData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxDistortionDefinitionData {
    pub distortion: Option<f32>,
    pub distortion_mode: Option<u8>,
    pub normal_map_texture: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterLegacySimple {
    pub rotation: Option<ValueFloat>,
    pub birth_scale: Option<ValueFloat>,
    pub orientation: Option<u8>,
    pub scale_bias: Option<Vec2>,
    pub scale: Option<ValueFloat>,
    pub birth_rotational_velocity: Option<ValueFloat>,
    pub uv_scroll_rate: Option<Vec2>,
    pub birth_rotation: Option<ValueFloat>,
    pub particle_bind: Option<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimeMaterialDriver {
    pub loop_duration: Option<f32>,
    pub loop_time_as_fraction: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionMultikillSize {
    pub m_multikill_size: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameCalculation {
    pub tooltip_only: Option<bool>,
    pub m_simple_tooltip_calculation_display: Option<u8>,
    pub m_multiplier: Option<StatBySubPartCalculationPartMSubpart>,
    pub m_precision: Option<i32>,
    pub m_display_as_percent: Option<bool>,
    pub m_formula_parts: Option<Vec<StatBySubPartCalculationPartMSubpart>>,
    pub m_expanded_tooltip_calculation_display: Option<u8>,
    pub result_modifier: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionWinningTeam {
    pub is_same_team: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x4a70b12c {
    pub unk_0x9a676645: Option<u32>,
    pub augment_group: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OverrideAttackTimeData {
    pub m_cast_time_percent: Option<f32>,
    pub m_total_attack_time_secs: Option<GameCalculation>,
    pub set_override_attack_delay: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x3c52b66 {
    pub text: Option<u32>,
    pub icon: Option<u32>,
    pub start_time: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedVector2fVariableData {
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub values: Option<Vec<Vec2>>,
    pub times: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x25e3f5d0 {
    pub definition: Unk0xf775806c,
    pub transform: Mat4,
    pub name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9e9e2e5c {
    pub unk_0x137cf12a: u32,
    pub source_object: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxColorOverLifeMaterialDriver {
    pub colors: VfxAnimatedColorVariableData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackFormationData {
    pub formation_positions: Vec<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x1003c990 {
    pub texture_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasNNearbyUnitsRequirement {
    pub m_range: f32,
    pub m_units_requirements: Box<Vec<SpellDataResourceMCastRequirementsCaster>>,
    pub m_distance_type: Option<u32>,
    pub m_units_required: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceListElement {
    pub name_override: Option<String>,
    pub multiplier: Option<f32>,
    pub type_index: Option<i32>,
    pub r#type: String,
    pub style: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OverrideAutoAttackCastTimeData {
    pub m_override_autoattack_cast_time_calculation: GameCalculation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveAttachedMesh {
    pub m_mesh: Option<VfxMeshDefinitionData>,
    pub use_avatar_specific_submesh_mask: Option<bool>,
    pub align_yaw_to_camera: Option<bool>,
    pub align_pitch_to_camera: Option<bool>,
    pub unk_0x6aec9e7a: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueVector2 {
    pub dynamics: VfxAnimatedVector2fVariableData,
    pub constant_value: Option<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatStoneData {
    pub m_name_tra_key: String,
    pub milestones: Vec<u64>,
    pub triggered_from_script: Option<bool>,
    pub stone_name: String,
    pub events_to_track: Vec<StatStoneEventToTrack>,
    pub catalog_entry: CatalogEntry,
    pub epic_stat_stone: Option<bool>,
    pub m_description_tra_key: String,
    pub category: u32,
    pub data_collection_only: Option<bool>,
    pub is_retired: Option<bool>,
    pub tracking_type: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbilityResourceSlotInfo {
    pub ar_base_factor_regen: Option<f32>,
    pub ar_allow_max_value_to_be_overridden: Option<bool>,
    pub ar_negative_spacer: Option<bool>,
    pub ar_override_small_pip_name: Option<String>,
    pub hide_empty_pips: Option<bool>,
    pub visibility_flags: Option<u8>,
    pub ar_type: Option<u8>,
    pub ar_per_level: Option<f32>,
    pub ar_max_segments: Option<i32>,
    pub ar_increments: Option<f32>,
    pub ar_base_static_regen: Option<f32>,
    pub ar_is_shown: Option<bool>,
    pub ar_base: Option<f32>,
    pub ar_override_large_pip_name: Option<String>,
    pub ar_display_as_pips: Option<bool>,
    pub ar_regen_per_level: Option<f32>,
    pub ar_override_empty_pip_name: Option<String>,
    pub ar_override_spacer_name: Option<String>,
    pub ar_has_regen_text: Option<bool>,
    pub unk_0x4eb6a404: Option<u8>,
    pub ar_override_medium_pip_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterToolData {
    pub map_ai_presence: Option<HashMap<u32, ToolAiPresence>>,
    pub base_attack_speed_bonus: Option<f32>,
    pub defense_rank: Option<i32>,
    pub spell_data: Option<Vec<ToolSpellDesc>>,
    pub bot_enabled_mm: Option<bool>,
    pub pass_lev1_desc: Option<Vec<String>>,
    pub rec_items: Option<Vec<String>>,
    pub champion_id: Option<i32>,
    pub sound: Option<ToolSoundData>,
    pub tutorial_rec_items: Option<Vec<String>>,
    pub description: Option<String>,
    pub bot_enabled: Option<bool>,
    pub cast_shadows: Option<bool>,
    pub bot_default_spell2: Option<String>,
    pub level_spell_effectiveness: Option<f32>,
    pub roles: Option<String>,
    pub unk_0xaa75da9d: Option<bool>,
    pub base_spell_effectiveness: Option<f32>,
    pub classification: Option<String>,
    pub chasing_attack_range_percent: Option<f32>,
    pub weapon_material_crit: Option<String>,
    pub search_tags_secondary: Option<String>,
    pub difficulty_rank: Option<i32>,
    pub passive_data: Option<Vec<ToolPassiveData>>,
    pub attack_speed: Option<f32>,
    pub tips3: Option<String>,
    pub bot_default_spell1: Option<String>,
    pub search_tags: Option<String>,
    pub lore2: Option<String>,
    pub attack_rank: Option<i32>,
    pub post_attack_move_delay: Option<f32>,
    pub inherits: Option<ToolInheritsData>,
    pub par_fade_color: Option<String>,
    pub magic_rank: Option<i32>,
    pub alternate_forms: Option<Vec<ToolAlternateForm>>,
    pub soul_given_on_death: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PercentageOfBuffNameElapsed {
    pub coefficient: f32,
    pub buff_name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LerpVec4LogicDriver {
    pub turn_on_time_sec: Option<f32>,
    pub turn_off_time_sec: Option<f32>,
    pub bool_driver: Box<BlendingSwitchMaterialDriverMDefaultValue>,
    pub off_value: Option<Vec4>,
    pub on_value: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnableLookAtEventData {
    pub m_name: Option<u32>,
    pub m_end_frame: Option<f32>,
    pub m_start_frame: Option<f32>,
    pub m_lock_current_values: Option<bool>,
    pub m_enable_look_at: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xba138ae3 {
    pub name: u32,
    pub transform: Mat4,
    pub definition: Unk0xfde6a2d7,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x2bfb084c {
    pub tags: Vec<Unk0xf6f4bb5f>,
    pub group_name: String,
    pub unk_0xec01928c: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiSpellData {
    pub m_block_level: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x51445de9 {
    pub value: Vec4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CcBehaviorData {
    pub cc_behavior: TargetingPriorityList,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionSkipTerrain {
    pub m_end_locator: DrawablePositionLocator,
    pub m_terrain_texture_name: String,
    pub m_base_texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSpellInfo {
    pub m_platform_enabled: Option<bool>,
    pub m_game_modes: Option<Vec<String>>,
    pub m_spell_id: Option<i32>,
    pub m_avatar_level_required: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionTeammateDeathsNearby {
    pub m_teammate_deaths: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinMeshDataPropertiesMaterialOverride {
    pub texture: Option<String>,
    pub submesh: String,
    pub material: Option<u32>,
    pub gloss_texture: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JointOrientationEventData {
    pub blend_data: Unk0x125a3586,
    pub m_end_frame: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Cast {
    pub roll_for_critical_hit_result: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WallFollowMovement {
    pub m_start_bone_name: String,
    pub m_use_ground_height_at_target: bool,
    pub m_infer_direction_from_facing_if_needed: bool,
    pub m_wall_offset: f32,
    pub m_wall_search_radius: f32,
    pub m_stop_halfway_around: bool,
    pub m_speed: f32,
    pub m_wall_length: f32,
    pub m_tracks_target: bool,
    pub use_point_smoothing: Option<bool>,
    pub m_counter_clockwise: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DecelToLocationMovement {
    pub m_project_target_to_cast_range: bool,
    pub m_tracks_target: bool,
    pub m_target_bone_name: Option<String>,
    pub m_initial_speed: f32,
    pub m_target_height_augment: Option<f32>,
    pub m_use_height_offset_at_end: bool,
    pub m_max_speed: f32,
    pub m_acceleration: f32,
    pub m_start_bone_name: Option<String>,
    pub m_min_speed: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicMaterialStaticSwitch {
    pub enabled: Option<bool>,
    pub driver: BlendingSwitchMaterialDriverMDefaultValue,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionIsAlly {
    pub m_is_ally: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemRecommendationChoices {
    pub m_choices: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceSpell {
    pub m_loc_keys: Option<HashMap<String, String>>,
    pub m_format: u32,
    pub enable_extended_tooltip: Option<bool>,
    pub m_lists: Option<HashMap<String, TooltipInstanceList>>,
    pub m_object_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GdsMapObjectBannerInfo {
    pub banner_data: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChildMapVisibilityController {
    pub parents: Vec<u32>,
    pub parent_mode: Option<u32>,
    pub path_hash: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionCharacterMetadata {
    pub m_data: String,
    pub m_category: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AllTrueMaterialDriver {
    pub m_drivers: Box<Option<Vec<BlendingSwitchMaterialDriverMDefaultValue>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorTypeGlobal {
    pub m_is_floating: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionSpellLevel {
    pub m_spell_level: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasNNearbyVisibleUnitsRequirement {
    pub m_units_required: u32,
    pub m_units_requirements: Box<Vec<SpellDataResourceMCastRequirementsCaster>>,
    pub m_range: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xc406a533 {
    pub name: u32,
    pub unk_0x27639032: u8,
    pub default_visible: bool,
    pub path_hash: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionBuffCounterSetToZeroAfterLimitReached {
    pub m_counter_highest_reached: u8,
    pub m_buff: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChampionRuneRecommendationsContext {}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemRecommendationContextList {
    pub m_all_starting_item_ids: HashMap<u32, ItemRecommendationItemList>,
    pub m_contexts: Vec<ItemRecommendationContext>,
    pub m_all_recommendable_item_ids: HashMap<u32, ItemRecommendationItemList>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ColorChooserMaterialDriver {
    pub m_bool_driver: Box<BlendingSwitchMaterialDriverMDefaultValue>,
    pub m_color_on: Option<Vec4>,
    pub m_color_off: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwitchMaterialDriverElement {
    pub m_condition: BlendingSwitchMaterialDriverMDefaultValue,
    pub m_value: BlendingSwitchMaterialDriverMDefaultValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LockRootOrientationEventData {
    pub blend_out_time: Option<f32>,
    pub m_start_frame: Option<f32>,
    pub m_end_frame: Option<f32>,
    pub joint_name: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SecondaryResourceDisplayFractional {}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueVector3 {
    pub dynamics: VfxAnimatedVector3fVariableData,
    pub constant_value: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlexValueFloat {
    pub m_flex_id: Option<u32>,
    pub m_value: Option<ValueFloat>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecSpellRankUpInfoList {
    pub rec_spell_rank_up_infos: Vec<RecSpellRankUpInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinAudioProperties {
    pub bank_units: Option<Vec<BankUnit>>,
    pub tag_event_list: Option<Vec<String>>,
    pub plays_vo: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MissileSpecification {
    pub missile_group_spawners: Option<Vec<MissileGroupSpawnerSpec>>,
    pub vertical_facing: Option<MissileSpecificationVerticalFacing>,
    pub unk_0xc195fba6: Option<bool>,
    pub height_solver: Option<MissileSpecificationHeightSolver>,
    pub visibility_component: Option<MissileSpecificationVisibilityComponent>,
    pub behaviors: Option<Vec<MissileSpecificationBehaviors>>,
    pub m_missile_width: Option<f32>,
    pub movement_component: MissileSpecificationMovementComponent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemRecommendationItemList {
    pub m_item_list: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValueFloat {
    pub constant_value: Option<f32>,
    pub dynamics: Option<VfxAnimatedFloatVariableData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ByCharLevelInterpolationCalculationPart {
    pub m_end_value: Option<f32>,
    pub m_scale_past_default_max_level: Option<bool>,
    pub m_start_value: Option<f32>,
    pub m_scale_by_stat_progression_multiplier: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xdb26669a {
    pub unk_0x90373b91: Option<Vec<Unk0xc2afdb3d>>,
    pub unk_0x19a9984e: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InhibitorWaveBehavior {
    pub spawn_count_per_inhibitor_down: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TriggerFromScript {
    pub m_trigger_name: String,
    pub m_delay: Option<f32>,
    pub m_actions: Vec<TriggerOnDelayMActions>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BasicSkinAugment {
    pub m_name_tra_key: Option<String>,
    pub m_description_tra_key: Option<String>,
    pub catalog_entry: CatalogEntry,
    pub modifiers: Vec<BasicSkinAugmentModifiers>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FadeEventData {
    pub m_name: Option<u32>,
    pub m_start_frame: Option<f32>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub m_target_alpha: f32,
    pub m_time_to_fade: Option<f32>,
    pub m_end_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CastOnHit {
    pub roll_for_critical_hit_result: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xdd661aab {
    pub override_params: Option<Unk0xc7e628b9>,
    pub unk_0x77cff90e: Unk0xd91a223,
    pub unk_0xe4ecb00c: Unk0xfb16e4be,
    pub unk_0xda1ee5bc: bool,
    pub trigger_spells: Vec<u32>,
    pub unk_0x96e77860: u32,
    pub unk_0x8f7842e4: Vec<Unk0x55f6bf86>,
    pub unk_0x6cd45762: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TextureResource {
    pub texture_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ByCharLevelFormulaCalculationPart {
    pub m_values: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x75e34c40 {
    pub unk_0x1dcc5270: Vec<Unk0xd5c9eb1>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolPassiveData {
    pub effect: Option<Vec<String>>,
    pub name: Option<String>,
    pub level: Option<Vec<i32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionCone {
    pub start_locator: Option<DrawablePositionLocator>,
    pub cone_angle_degrees: Option<f32>,
    pub end_locator: DrawablePositionLocator,
    pub fallback_direction: Option<u32>,
    pub cone_range: Option<f32>,
    pub texture_cone_override_name: Option<String>,
    pub cone_follows_end: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x8958fee2 {
    pub unk_0x79a2e7aa: Option<f32>,
    pub unk_0xffcbd9e2: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub m_can_complete_cast_without_target: Option<bool>,
    pub unk_0xfb5bbd7: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapCubemapProbe {
    pub transform: Mat4,
    pub cubemap_probe_path: String,
    pub cubemap_probe_scale: f32,
    pub name: String,
    pub m_visibility_flags: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellLevelUpInfo {
    pub m_requirements: Vec<SpellRankUpRequirements>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BankUnit {
    pub name: String,
    pub bank_path: Option<Vec<String>>,
    pub voice_over: Option<bool>,
    pub events: Option<Vec<String>>,
    pub asynchrone: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScriptDataObject {
    pub m_required_constants_group: u32,
    pub m_constants: HashMap<String, ScriptDataObjectMConstants>,
    pub m_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPaletteDefinitionData {
    pub palette_v_animation_curve: Option<ValueFloat>,
    pub palette_texture: Option<String>,
    pub palette_selector: Option<ValueVector3>,
    pub pallete_src_mix_color: Option<ValueColor>,
    pub palette_count: Option<i32>,
    pub palette_u_animation_curve: Option<ValueFloat>,
    pub palette_texture_address_mode: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SineMaterialDriver {
    pub m_scale: Option<f32>,
    pub m_frequency: Option<f32>,
    pub m_driver: Box<BlendingSwitchMaterialDriverMDefaultValue>,
    pub m_bias: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemRecommendationMatrixRow {
    pub m_choices_map: Option<HashMap<String, ItemRecommendationChoices>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HermiteSplineInfo {
    pub m_start_position_offset: Option<Vec3>,
    pub m_use_missile_position_as_origin: Option<bool>,
    pub m_control_point2: Option<Vec3>,
    pub m_control_point1: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionLine {
    pub minimum_displayed_range: Option<f32>,
    pub range_growth_max: Option<FloatPerSpellLevel>,
    pub range_growth_start_time: Option<FloatPerSpellLevel>,
    pub facing_line: Option<bool>,
    pub arrow_size: Option<f32>,
    pub end_locator: Option<DrawablePositionLocator>,
    pub override_base_range: Option<FloatPerSpellLevel>,
    pub fade: Option<bool>,
    pub fallback_direction: Option<u32>,
    pub use_global_line_indicator: Option<bool>,
    pub m_center_arrow_to_end_point: Option<bool>,
    pub line_width: Option<FloatPerSpellLevel>,
    pub m_fade_behavior: Option<FadeOverTimeBehavior>,
    pub texture_target_override_name: Option<String>,
    pub indicator_type: Option<TargeterDefinitionLineIndicatorType>,
    pub line_stops_at_end_position: Option<bool>,
    pub texture_base_override_name: Option<String>,
    pub texture_target_max_grow_name: Option<String>,
    pub always_draw: Option<bool>,
    pub start_locator: Option<DrawablePositionLocator>,
    pub texture_base_max_grow_name: Option<String>,
    pub range_growth_duration: Option<FloatPerSpellLevel>,
    pub max_angle: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionKillCount {
    pub m_total_kills: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionBuffCounterChanged {
    pub buff: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParametricMovementTypeAngleFromTarget {}

#[derive(Serialize, Deserialize, Debug, Clone, TypePath, Asset)]
#[serde(rename_all = "camelCase")]
pub struct CharacterRecord {
    pub local_gold_split_with_last_hitter: Option<bool>,
    pub health_bar_height: Option<f32>,
    pub self_cb_champ_specific_health_suffix: Option<String>,
    pub extra_spells: Option<Vec<String>>,
    pub base_crit_chance: Option<f32>,
    pub useable_data: Option<UseableData>,
    pub base_spell_block: Option<f32>,
    pub local_exp_given_on_death: Option<f32>,
    pub attack_speed_ratio: Option<f32>,
    pub flags: Option<u32>,
    pub untargetable_spawn_time: Option<f32>,
    pub perception_bubble_radius: Option<f32>,
    pub base_hp: Option<f32>,
    pub gold_given_on_death: Option<f32>,
    pub passive_spell: Option<String>,
    pub on_kill_event_for_spectator: Option<u32>,
    pub rec_spell_rank_up_info_list: Option<RecSpellRankUpInfoList>,
    pub spells: Option<Vec<u32>>,
    pub unk_0x43135375: Option<f32>,
    pub override_gameplay_collision_radius: Option<f32>,
    pub hover_line_indicator_width: Option<f32>,
    pub passive_tool_tip: Option<String>,
    pub first_acquisition_range: Option<f32>,
    pub silhouette_attachment_anim: Option<String>,
    pub m_preferred_perk_style: Option<u32>,
    pub m_character_passive_buffs: Option<Vec<CharacterPassiveData>>,
    pub hover_line_indicator_width_minimap: Option<f32>,
    pub hover_line_indicator_base_texture_name: Option<String>,
    pub secondary_ability_resource: Option<AbilityResourceSlotInfo>,
    pub local_gold_given_on_death: Option<f32>,
    pub enemy_tooltip: Option<String>,
    pub crit_per_level: Option<f32>,
    pub selection_radius: Option<f32>,
    pub base_armor: Option<f32>,
    pub highlight_healthbar_icons: Option<bool>,
    pub weapon_materials: Option<Vec<String>>,
    pub friendly_ux_override_exclude_tags_string: Option<String>,
    pub passive1_icon_name: Option<String>,
    pub unk_0xc1984296: Option<Vec<u32>>,
    pub primary_ability_resource: Option<AbilityResourceSlotInfo>,
    pub area_indicator_max_distance: Option<f32>,
    pub m_education_tool_data: Option<ToolEducationData>,
    pub hover_indicator_radius_minimap: Option<f32>,
    pub spell_block_per_level: Option<f32>,
    pub evolution_data: Option<EvolutionDescription>,
    pub m_use_cc_animations: Option<bool>,
    pub m_character_name: String,
    pub crit_damage_multiplier: Option<f32>,
    pub target_laser_effects: Option<TargetLaserComponentEffects>,
    pub extra_attacks: Option<Vec<AttackSlotData>>,
    pub m_character_calculations: Option<HashMap<u32, GameCalculation>>,
    pub hp_regen_per_level: Option<f32>,
    pub unk_0xdd661aab: Option<Unk0x280745b1>,
    pub unk_0x9836cd87: Option<u8>,
    pub m_abilities: Option<Vec<u32>>,
    pub hover_indicator_radius: Option<f32>,
    pub attack_speed_per_level: Option<f32>,
    pub disguise_minimap_icon_override: Option<String>,
    pub minion_flags: Option<u32>,
    pub pack_manager_data: Option<PackManagerData>,
    pub hp_per_level: Option<f32>,
    pub self_champ_specific_health_suffix: Option<String>,
    pub friendly_ux_override_include_tags_string: Option<String>,
    pub char_audio_name_override: Option<String>,
    pub global_gold_given_on_death: Option<f32>,
    pub death_time: Option<f32>,
    pub platform_enabled: Option<bool>,
    pub attack_speed: Option<f32>,
    pub critical_attack: Option<String>,
    pub launch_area_data: Option<LaunchAreaData>,
    pub damage_per_level: Option<f32>,
    pub basic_attack: Option<AttackSlotData>,
    pub gold_radius: Option<f32>,
    pub experience_radius: Option<f32>,
    pub area_indicator_target_distance: Option<f32>,
    pub disabled_target_laser_effects: Option<TargetLaserComponentEffects>,
    pub selection_height: Option<f32>,
    pub enemy_champ_specific_health_suffix: Option<String>,
    pub outline_b_box_expansion: Option<f32>,
    pub global_exp_given_on_death: Option<f32>,
    pub purchase_identities: Option<Vec<u32>>,
    pub base_static_hp_regen: Option<f32>,
    pub m_fallback_character_name: Option<String>,
    pub unit_tags_string: Option<String>,
    pub m_adaptive_force_to_ability_power_weight: Option<f32>,
    pub ally_champ_specific_health_suffix: Option<String>,
    pub character_tool_data: Option<CharacterToolData>,
    pub base_factor_hp_regen: Option<f32>,
    pub on_kill_event: Option<u32>,
    pub pathfinding_collision_radius: Option<f32>,
    pub significance: Option<f32>,
    pub acquisition_range: Option<f32>,
    pub passive_lua_name: Option<String>,
    pub tower_targeting_priority_boost: Option<f32>,
    pub joint_for_anim_adjusted_selection: Option<String>,
    pub occluded_unit_selectable_distance: Option<f32>,
    pub record_as_ward: Option<bool>,
    pub armor_per_level: Option<f32>,
    pub death_event_listening_radius: Option<f32>,
    pub m_perk_replacements: Option<PerkReplacementList>,
    pub health_bar_full_parallax: Option<bool>,
    pub perception_bounding_box_size: Option<Vec3>,
    pub base_move_speed: Option<f32>,
    pub spell_level_up_info: Option<Vec<SpellLevelUpInfo>>,
    pub area_indicator_texture_size: Option<f32>,
    pub hit_fx_scale: Option<f32>,
    pub use_riot_relationships: Option<bool>,
    pub m_character_passive_spell: Option<u32>,
    pub hover_indicator_minimap_override: Option<String>,
    pub hover_indicator_texture_name: Option<String>,
    pub minimap_icon_override: Option<String>,
    pub attack_auto_interrupt_percent: Option<f32>,
    pub friendly_tooltip: Option<String>,
    pub spell_names: Option<Vec<String>>,
    pub passive_range: Option<f32>,
    pub hover_line_indicator_target_texture_name: Option<String>,
    pub exp_given_on_death: Option<f32>,
    pub on_kill_event_steal: Option<u32>,
    pub area_indicator_min_radius: Option<f32>,
    pub m_client_side_item_inventory: Option<Vec<u32>>,
    pub m_ability_slot_cc: Option<Vec<i32>>,
    pub hover_indicator_rotate_to_player: Option<bool>,
    pub name: Option<String>,
    pub crit_attacks: Option<Vec<AttackSlotData>>,
    pub treat_auto_attacks_as_normal_spells: Option<TreatAutoAttacksAsNormalSpells>,
    pub minion_score_value: Option<f32>,
    pub passive_name: Option<String>,
    pub wake_up_range: Option<f32>,
    pub area_indicator_radius: Option<f32>,
    pub base_damage: Option<f32>,
    pub par_name: Option<String>,
    pub unk_0x6854087e: Option<Vec<Unk0x47f13ab0>>,
    pub friendly_ux_override_team: Option<u32>,
    pub unk_0xc5c48b41: Option<u8>,
    pub area_indicator_min_distance: Option<f32>,
    pub attack_range: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MinMaterialDriver {
    pub m_drivers: Box<Vec<BlendingSwitchMaterialDriverMDefaultValue>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FadeToExplicitValueBehavior {
    pub m_alpha: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TftCharacterRecord {
    pub flags: u32,
    pub m_move_interval: Option<f32>,
    pub primary_ability_resource: AbilityResourceSlotInfo,
    pub m_uses_ability_power: bool,
    pub omit_from_match_history: bool,
    pub omit_from_combat_recap: bool,
    pub m_character_name: String,
    pub m_mana_per_attack: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParametricMovement {
    pub m_offset_initial_target_height: f32,
    pub m_target_height_augment: f32,
    pub parametric_movement_type: ParametricMovementTypeAngleFromTarget,
    pub m_start_bone_name: String,
    pub movement_entries: Vec<ParametricMovementEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xe07edfa4 {
    pub default_visible: Option<bool>,
    pub name: u32,
    pub path_hash: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackManagerData {
    pub attack_move_target_forgiveness_range: Option<f32>,
    pub unk_0xb97a9b92: Option<bool>,
    pub unk_0x377491e8: PackManagerDataUnknownEnumField,
    pub leash_distance: Option<f32>,
    pub follower_crossover_animation: u32,
    pub ui_target_forgiveness_range: Option<f32>,
    pub rank_to_formation_map: Option<HashMap<u32, PackFormationData>>,
    pub on_leader_move_follower_animation: u32,
    pub buff_overrides: Option<Vec<u32>>,
    pub order_trailing_delay: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetingRangeValue {
    pub range: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberCalculationPart {
    pub m_number: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xee18a47b {
    pub unk_0x589a59c: u32,
    pub unk_0xb65bc23: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinCharacterMetaDataPropertiesSpawningSkinOffset {
    pub tag: String,
    pub offset: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionWall {
    pub thickness: FloatPerSpellLevel,
    pub length: FloatPerSpellLevel,
    pub center_locator: DrawablePositionLocator,
    pub texture_wall_override_name: Option<String>,
    pub wall_rotation: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasBuffCastRequirement {
    pub m_from_anyone: Option<bool>,
    pub m_buff_name: u32,
    pub m_invert_result: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xcdb1c8f6 {
    pub unk_0x6355dd6f: Vec<Unk0x6355dd6f>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionNeutralCampId {
    pub m_camp_id: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasAllSubRequirementsCastRequirement {
    pub m_sub_requirements: Box<Vec<SpellDataResourceMCastRequirementsCaster>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CastOnMovementComplete {
    pub roll_for_critical_hit_result: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxTextureMultDefinitionData {
    pub birth_uv_scroll_rate_mult: Option<ValueVector2>,
    pub flex_birth_uv_scroll_rate_mult: Option<FlexValueVector2>,
    pub texture_mult_filp_u: Option<bool>,
    pub uv_scale_mult: Option<ValueVector2>,
    pub texture_mult: Option<String>,
    pub uv_scroll_clamp_mult: Option<bool>,
    pub uv_transform_center_mult: Option<Vec2>,
    pub emitter_uv_scroll_rate_mult: Option<Vec2>,
    pub is_random_start_frame_mult: Option<bool>,
    pub uv_scroll_alpha_mult: Option<bool>,
    pub tex_address_mode_mult: Option<u8>,
    pub birth_uv_offset_mult: Option<ValueVector2>,
    pub particle_integrated_uv_rotate_mult: Option<IntegratedValueFloat>,
    pub birth_uv_rotate_rate_mult: Option<ValueFloat>,
    pub particle_integrated_uv_scroll_mult: Option<IntegratedValueVector2>,
    pub tex_div_mult: Option<Vec2>,
    pub uv_rotation_mult: Option<ValueFloat>,
    pub texture_mult_filp_v: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualActionPlayAnimation {
    pub m_play_as_emote: bool,
    pub m_hashed_situation_trigger: Option<u32>,
    pub m_hashed_animation_name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnimationFractionDynamicMaterialFloatDriver {
    pub m_animation_name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionArc {
    pub thickness_offset: Option<f32>,
    pub constraint_range: FloatPerSpellLevel,
    pub texture_arc_override_name: Option<String>,
    pub end_locator: DrawablePositionLocator,
    pub override_radius: FloatPerSpellLevel,
    pub start_locator: Option<DrawablePositionLocator>,
    pub is_constrained_to_range: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinUpgradeData {
    pub m_gear_skin_upgrades: Option<Vec<u32>>,
    pub skin_augment_categories: Option<SkinAugmentCategories>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xc2afdb3d {
    pub icon: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ESportTeamEntry {
    pub league_name: Option<String>,
    pub texture_name: Option<String>,
    pub team_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionCharacterSkinId {
    pub m_skin_i_ds: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BorderPropertyData {
    pub border_path: String,
    pub border_treatment: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9d9f60d2 {
    pub unk_0xde46f1d8: Option<String>,
    pub unk_0x397fe037: Option<bool>,
    pub character_record: String,
    pub tags: Option<Vec<u32>>,
    pub unk_0xf1d3a034: Option<bool>,
    pub skin: String,
    pub team: Option<u32>,
    pub unk_0xdbde2288: Option<Vec<Unk0x82cab1b3>>,
    pub r#type: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedTimeSplineMovement {
    pub m_spline_info: HermiteSplineInfo,
    pub m_target_height_augment: Option<f32>,
    pub m_target_bone_name: Option<String>,
    pub m_start_bone_name: String,
    pub m_travel_time: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x3f04641e {
    pub camp_name: String,
    pub unk_0x7511a599: Option<Vec<Unk0xdb26669a>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TakeCamp {
    pub camp: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAlphaErosionDefinitionData {
    pub erosion_drive_source: Option<u8>,
    pub use_linger_erosion_drive_curve: Option<bool>,
    pub erosion_map_name: Option<String>,
    pub erosion_feather_in: Option<f32>,
    pub erosion_slice_width: Option<f32>,
    pub erosion_map_address_mode: Option<u8>,
    pub erosion_feather_out: Option<f32>,
    pub erosion_map_channel_mixer: Option<ValueColor>,
    pub erosion_drive_curve: Option<ValueFloat>,
    pub linger_erosion_drive_curve: Option<ValueFloat>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpringPhysicsEventData {
    pub spring_to_affect: Option<u32>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub blend_out_time: Option<f32>,
    pub m_end_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UvScaleBiasFromAnimationDynamicMaterialDriver {
    pub m_sub_mesh_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatStoneSet {
    pub catalog_entry: CatalogEntry,
    pub name: String,
    pub stat_stones: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionSpellBuffName {
    pub spell_buff: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasGearDynamicMaterialBoolDriver {
    pub m_gear_index: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IsCastingBoolDriver {
    pub spell_slot: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialShaderSamplerDef {
    pub texture_path: Option<String>,
    pub address_w: Option<u32>,
    pub texture_name: String,
    pub address_v: Option<u32>,
    pub address_u: Option<u32>,
    pub uncensored_textures: Option<HashMap<u32, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterDefinitionData {
    pub uv_scale: Option<ValueVector2>,
    pub uv_scroll_clamp: Option<bool>,
    pub pass: Option<i16>,
    pub birth_color: Option<ValueColor>,
    pub lifetime: Option<f32>,
    pub blend_mode: Option<u8>,
    pub flex_birth_uv_offset: Option<FlexValueVector2>,
    pub legacy_simple: Option<VfxEmitterLegacySimple>,
    pub alpha_ref: Option<u8>,
    pub rotation0: Option<IntegratedValueVector3>,
    pub emission_mesh_name: Option<String>,
    pub uv_mode: Option<u8>,
    pub falloff_texture: Option<String>,
    pub flex_birth_uv_scroll_rate: Option<FlexValueVector2>,
    pub is_texture_pixelated: Option<bool>,
    pub write_alpha_only: Option<bool>,
    pub disable_backface_cull: Option<bool>,
    pub maximum_rate_by_velocity: Option<f32>,
    pub soft_particle_params: Option<VfxSoftParticleDefinitionData>,
    pub num_frames: Option<u16>,
    pub particle_color_texture: Option<String>,
    pub flex_shape_definition: Option<VfxFlexShapeDefinitionData>,
    pub sort_emitters_by_pos: Option<bool>,
    pub translation_override: Option<Vec3>,
    pub uv_parallax_scale: Option<f32>,
    pub particles_share_random_value: Option<bool>,
    pub birth_velocity: Option<ValueVector3>,
    pub unk_0xcb13aff1: Option<f32>,
    pub is_emitter_space: Option<bool>,
    pub mesh_render_flags: Option<u8>,
    pub texture: Option<String>,
    pub flex_birth_velocity: Option<FlexValueVector3>,
    pub spawn_shape: Option<VfxEmitterDefinitionDataSpawnShape>,
    pub emitter_name: Option<String>,
    pub birth_uv_scroll_rate: Option<ValueVector2>,
    pub birth_rotational_acceleration: Option<ValueVector3>,
    pub color_look_up_type_x: Option<u8>,
    pub particle_is_local_orientation: Option<bool>,
    pub texture_flip_u: Option<bool>,
    pub emission_surface_definition: Option<VfxEmissionSurfaceData>,
    pub censor_modulate_value: Option<Vec4>,
    pub color_look_up_offsets: Option<Vec2>,
    pub direction_velocity_min_scale: Option<f32>,
    pub time_active_during_period: Option<f32>,
    pub tex_address_mode_base: Option<u8>,
    pub is_single_particle: Option<bool>,
    pub acceleration: Option<ValueVector3>,
    pub has_variable_start_time: Option<bool>,
    pub misc_render_flags: Option<u8>,
    pub post_rotate_orientation_axis: Option<Vec3>,
    pub chance_to_not_exist: Option<f32>,
    pub offset_life_scaling_symmetry_mode: Option<u8>,
    pub velocity: Option<ValueVector3>,
    pub is_direction_oriented: Option<bool>,
    pub drag: Option<ValueVector3>,
    pub emission_mesh_scale: Option<f32>,
    pub is_uniform_scale: Option<bool>,
    pub rate_by_velocity_function: Option<ValueVector2>,
    pub is_following_terrain: Option<bool>,
    pub scale0: Option<ValueVector3>,
    pub unk_0xd1ee8634: Option<bool>,
    pub texture_flip_v: Option<bool>,
    pub audio: Option<VfxEmitterAudio>,
    pub palette_definition: Option<VfxPaletteDefinitionData>,
    pub birth_rotation0: Option<ValueVector3>,
    pub distortion_definition: Option<VfxDistortionDefinitionData>,
    pub time_before_first_emission: Option<f32>,
    pub color_render_flags: Option<u8>,
    pub birth_frame_rate: Option<ValueFloat>,
    pub birth_acceleration: Option<ValueVector3>,
    pub tex_div: Option<Vec2>,
    pub stencil_ref: Option<u8>,
    pub depth_bias_factors: Option<Vec2>,
    pub slice_technique_range: Option<f32>,
    pub custom_material: Option<VfxMaterialDefinitionData>,
    pub bind_weight: Option<ValueFloat>,
    pub child_particle_set_definition: Option<VfxChildParticleSetDefinitionData>,
    pub stencil_mode: Option<u8>,
    pub emitter_linger: Option<f32>,
    pub is_random_start_frame: Option<bool>,
    pub importance: Option<u8>,
    pub particle_uv_scroll_rate: Option<IntegratedValueVector2>,
    pub flex_scale_birth_scale: Option<FlexTypeFloat>,
    pub field_collection_definition: Option<VfxFieldCollectionDefinitionData>,
    pub birth_drag: Option<ValueVector3>,
    pub period: Option<f32>,
    pub birth_rotational_velocity0: Option<ValueVector3>,
    pub has_post_rotate_orientation: Option<bool>,
    pub rate: Option<ValueFloat>,
    pub birth_uv_rotate_rate: Option<ValueFloat>,
    pub render_phase_override: Option<u8>,
    pub filtering: Option<VfxEmitterFiltering>,
    pub particle_linger: Option<f32>,
    pub birth_orbital_velocity: Option<ValueVector3>,
    pub material_override_definitions: Option<Vec<VfxMaterialOverrideDefinitionData>>,
    pub offset_lifetime_scaling: Option<Vec3>,
    pub particle_uv_rotate_rate: Option<IntegratedValueFloat>,
    pub flex_rate: Option<FlexValueFloat>,
    pub disabled: Option<bool>,
    pub emitter_uv_scroll_rate: Option<Vec2>,
    pub uv_transform_center: Option<Vec2>,
    pub use_navmesh_mask: Option<bool>,
    pub frame_rate: Option<f32>,
    pub world_acceleration: Option<IntegratedValueVector3>,
    pub texture_mult: Option<VfxTextureMultDefinitionData>,
    pub is_ground_layer: Option<bool>,
    pub particle_lifetime: Option<ValueFloat>,
    pub linger: Option<VfxLingerDefinitionData>,
    pub uv_rotation: Option<ValueFloat>,
    pub flex_particle_lifetime: Option<FlexValueFloat>,
    pub color_look_up_scales: Option<Vec2>,
    pub is_local_orientation: Option<bool>,
    pub modulation_factor: Option<Vec4>,
    pub particle_linger_type: Option<u8>,
    pub color_look_up_type_y: Option<u8>,
    pub stencil_reference_id: Option<u32>,
    pub reflection_definition: Option<VfxReflectionDefinitionData>,
    pub rotation_override: Option<Vec3>,
    pub birth_uv_offset: Option<ValueVector2>,
    pub use_emission_mesh_normal_for_birth: Option<bool>,
    pub flex_instance_scale: Option<FlexTypeFloat>,
    pub alpha_erosion_definition: Option<VfxAlphaErosionDefinitionData>,
    pub birth_scale0: Option<ValueVector3>,
    pub color: Option<ValueColor>,
    pub primitive: Option<VfxEmitterDefinitionDataPrimitive>,
    pub is_rotation_enabled: Option<bool>,
    pub scale_override: Option<Vec3>,
    pub emitter_position: Option<ValueVector3>,
    pub flex_birth_rotational_velocity0: Option<FlexValueVector3>,
    pub does_lifetime_scale: Option<bool>,
    pub does_cast_shadow: Option<bool>,
    pub start_frame: Option<u16>,
    pub colorblind_visibility: Option<u8>,
    pub direction_velocity_scale: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFlexShapeDefinitionData {
    pub scale_emit_offset_by_bound_object_height: Option<f32>,
    pub flex_scale_emit_offset: Option<FlexTypeFloat>,
    pub flex_birth_translation: Option<FlexValueVector3>,
    pub scale_birth_scale_by_bound_object_radius: Option<f32>,
    pub scale_emit_offset_by_bound_object_size: Option<f32>,
    pub scale_birth_scale_by_bound_object_height: Option<f32>,
    pub scale_emit_offset_by_bound_object_radius: Option<f32>,
    pub scale_birth_translation_by_bound_object_size: Option<f32>,
    pub scale_birth_scale_by_bound_object_size: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAssetRemap {
    pub r#type: Option<u32>,
    pub old_asset: Option<u32>,
    pub new_asset: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConditionBoolClipData {
    pub sync_frame_on_change_anim: Option<bool>,
    pub m_flags: Option<u32>,
    pub dont_stomp_transition_clip: Option<bool>,
    pub m_true_condition_clip_name: u32,
    pub updater: ConditionBoolClipDataUpdater,
    pub m_play_anim_change_from_beginning: Option<bool>,
    pub m_change_animation_mid_play: Option<bool>,
    pub m_false_condition_clip_name: u32,
    pub m_child_anim_delay_switch_time: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetTypeFilter {
    pub minions_are_valid: Option<bool>,
    pub champions_are_valid: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UseAutoattackCastTimeData {
    pub m_use_cast_time_as_total_time: Option<bool>,
    pub m_autoattack_cast_time_calculation: Option<GameCalculation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerkReplacement {
    pub m_replace_target: u32,
    pub m_replace_with: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MinionUpgradeConfig {
    pub unk_0x726ae049: Option<f32>,
    pub armor_max: Option<f32>,
    pub armor_upgrade_growth: Option<f32>,
    pub gold_max: Option<f32>,
    pub hp_max_bonus: f32,
    pub damage_upgrade_late: Option<f32>,
    pub hp_upgrade_late: Option<f32>,
    pub damage_upgrade: Option<f32>,
    pub hp_upgrade: f32,
    pub damage_max: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolAiPresence {
    pub intro: Option<bool>,
    pub unk_0xb75b2ab8: Option<bool>,
    pub unk_0xb66d0e47: Option<bool>,
    pub easy: Option<bool>,
    pub unk_0x42ac598e: Option<bool>,
    pub hard: Option<bool>,
    pub medium: Option<bool>,
    pub unk_0x6175bb7b: Option<bool>,
    pub unk_0xca762bfc: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinCharacterDataPropertiesCharacterIdleEffect {
    pub bone_name: Option<String>,
    pub effect_key: Option<u32>,
    pub effect_name: Option<String>,
    pub position: Option<Vec3>,
    pub target_bone_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionRange {
    pub center_locator: Option<DrawablePositionLocator>,
    pub range_growth_max: Option<FloatPerSpellLevel>,
    pub override_base_range: Option<FloatPerSpellLevel>,
    pub has_max_grow_range: Option<bool>,
    pub texture_max_grow_name: Option<String>,
    pub use_caster_bounding_box: Option<bool>,
    pub texture_orientation: Option<u32>,
    pub texture_override_name: Option<String>,
    pub hide_with_line_indicator: Option<bool>,
    pub range_growth_start_time: Option<FloatPerSpellLevel>,
    pub m_fade_behavior: Option<TargeterDefinitionRangeMFadeBehavior>,
    pub range_growth_duration: Option<FloatPerSpellLevel>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxReflectionDefinitionData {
    pub reflection_fresnel: Option<f32>,
    pub fresnel_color: Option<Vec4>,
    pub reflection_opacity_glancing: Option<f32>,
    pub reflection_opacity_direct: Option<f32>,
    pub reflection_map_texture: Option<String>,
    pub reflection_fresnel_color: Option<Vec4>,
    pub fresnel: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbilityObject {
    pub m_type: Option<u8>,
    pub m_lifetime_manually_managed: Option<bool>,
    pub m_child_spells: Option<Vec<u32>>,
    pub m_root_spell: u32,
    pub m_name: String,
    pub ability_traits: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurveTheDifferenceHeightSolver {
    pub m_initial_target_height_offset: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xfe70e9c4 {
    pub unk_0x4e748038: Option<u8>,
    pub unk_0x3ef62dce: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinAnimationProperties {
    pub animation_graph_data: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasTypeAndStatusFlags {
    pub m_affects_type_flags: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EvolutionDescription {
    pub m_tooltips: Option<Vec<String>>,
    pub m_icon_names: Vec<String>,
    pub m_flags: Option<u32>,
    pub m_title: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ESportLeagueEntry {
    pub texture_name: Option<String>,
    pub league_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapLocator {
    pub name: String,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RatioConversion {
    pub m_source_stat_output: Option<u8>,
    pub m_source_to_result_conversion_ratio: f32,
    pub m_source_stat_type: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveCameraSegmentBeam {
    pub m_beam: VfxBeamDefinitionData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x1aae122 {
    pub max_distance: f32,
    pub min_distance: f32,
    pub unk_0x7863785e: f32,
    pub max_offset_delta: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxChildIdentifier {
    pub effect: Option<u32>,
    pub effect_key: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedSpeedSplineMovement {
    pub m_target_bone_name: Option<String>,
    pub m_start_bone_name: Option<String>,
    pub m_use_height_offset_at_end: Option<bool>,
    pub m_spline_info: HermiteSplineInfo,
    pub m_speed: f32,
    pub m_target_height_augment: Option<f32>,
    pub m_use_missile_position_as_origin: Option<bool>,
    pub m_tracks_target: Option<bool>,
    pub m_offset_initial_target_height: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualSituation {
    pub m_rules: Option<Vec<ContextualRule>>,
    pub m_choose_random_valid_rule: Option<bool>,
    pub m_cool_down_time: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x634a3a64 {
    pub color: [u8; 4],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DestroyOnMovementComplete {
    pub m_delay: Option<i32>,
    pub render_particles_after_destroy: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GearSkinUpgrade {
    pub m_gear_data: GearData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xd178749c {
    pub definition: Unk0x429a2180,
    pub transform: Mat4,
    pub name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x280745b1 {
    pub params: Unk0xc7e628b9,
    pub unk_0x50aad250: Vec<Unk0xdd661aab>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemRecommendationOverride {
    pub m_core_items: Option<Vec<u32>>,
    pub m_force_override: Option<bool>,
    pub m_rec_item_ranges: Option<Vec<ItemRecOverrideRange>>,
    pub starting_item_bundles: Option<Vec<ItemRecommendationOverrideStartingItemBundle>>,
    pub m_override_contexts: Vec<ItemRecommendationOverrideContext>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterFiltering {
    pub keywords_excluded: Option<Vec<String>>,
    pub censor_policy: Option<u8>,
    pub keywords_required: Option<Vec<String>>,
    pub spectator_policy: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EsportsBannerData {
    pub team: u32,
    pub banner_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbilityResourceByCoefficientCalculationPart {
    pub m_coefficient: f32,
    pub m_stat_formula: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x1f1f50f2 {
    pub name: u32,
    pub definition: Unk0x8ad25772,
    pub transform: Mat4,
    pub unk_0xbbe68da1: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameFontDescription {
    pub name: String,
    pub resolution_data: u32,
    pub shadow_color: Option<[u8; 4]>,
    pub outline_color: [u8; 4],
    pub color: [u8; 4],
    pub fill_texture_name: String,
    pub type_data: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionCharacterDistance {
    pub m_distance: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlexTypeFloat {
    pub m_flex_id: Option<u32>,
    pub m_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TriggerOnMovementComplete {
    pub m_actions: Vec<TriggerOnDelayMActions>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasUnitTagsCastRequirement {
    pub m_unit_tags: ObjectTags,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JointSnapEventData {
    pub m_joint_name_to_override: Option<u32>,
    pub m_start_frame: Option<f32>,
    pub m_end_frame: Option<f32>,
    pub m_is_self_only: Option<bool>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub m_joint_name_to_snap_to: Option<u32>,
    pub offset: Option<Vec3>,
    pub m_name: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NavGridConfig {
    pub terrain_config: u32,
    pub region_groups: Vec<Unk0x2bfb084c>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapSunProperties {
    pub sun_direction: Vec3,
    pub fog_alternate_color: Vec4,
    pub ground_color: Vec4,
    pub light_map_color_scale: f32,
    pub fog_start_and_end: Vec2,
    pub sky_light_color: Vec4,
    pub fog_color: Vec4,
    pub horizon_color: Vec4,
    pub sky_light_scale: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetingForgivenessDefinitions {
    pub forgiveness_range: f32,
    pub override_affects_flags: Option<bool>,
    pub m_affects_type_override: Option<u32>,
    pub caster_forgiveness_definitions: Option<Vec<SameTeamCastRequirement>>,
    pub target_forgiveness_definitions: Option<Vec<SpellDataResourceMCastRequirementsCaster>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DelayedBoolMaterialDriver {
    pub m_bool_driver: Box<BlendingSwitchMaterialDriverMDefaultValue>,
    pub m_delay_off: Option<f32>,
    pub m_delay_on: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParametricPairData {
    pub m_value: Option<f32>,
    pub m_clip_name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xe7ee4f28 {
    pub unk_0xc19c58be: Option<HashMap<String, String>>,
    pub unk_0xa2cb8e03: Option<HashMap<String, u32>>,
    pub unk_0x7dd33afb: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimedVariableWaveBehavior {
    pub behaviors: Box<Vec<TimedWaveBehaviorInfo>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionSpellIsReady {
    pub m_spell_is_ready: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xe90af953 {
    pub unk_0xbe161d6e: u8,
    pub buff: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionCharacterName {
    pub m_characters: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MissileAttachedTargetingDefinition {
    pub m_line_end_texture_width: f32,
    pub m_line_texture_width: f32,
    pub m_line_end_texture_height: f32,
    pub m_end_position_type: u8,
    pub m_line_end_texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xe6147387 {
    pub unk_0xae1cbd5f: u8,
    pub orientation_source: Unk0x19da44b2,
    pub orientation_type: u8,
    pub unk_0xab2e032a: u8,
    pub unk_0xa57f0269: u8,
    pub joints: Vec<u32>,
    pub unk_0x1a30a486: bool,
    pub unk_0x420b233d: f32,
    pub default_on: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductOfSubPartsCalculationPart {
    pub m_part1: Box<StatBySubPartCalculationPartMSubpart>,
    pub m_part2: Box<StatBySubPartCalculationPartMSubpart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FaceTargetEventData {
    pub blend_in_time: Option<f32>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub y_rotation_degrees: Option<f32>,
    pub face_target: Option<u8>,
    pub m_name: Option<u32>,
    pub m_is_self_only: Option<bool>,
    pub blend_out_time: Option<f32>,
    pub m_end_frame: Option<f32>,
    pub m_start_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xc76c1b9a {
    pub modifiers: Vec<Unk0xc76c1b9aModifiers>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeMissileSpeed {
    pub m_speed_value: f32,
    pub trigger_only_once: Option<bool>,
    pub m_speed_change_type: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xb09016f6 {
    pub effect_tag: u32,
    pub effect_calculation: GameCalculation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEventDataPair {
    pub m_bone_name: Option<u32>,
    pub m_target_bone_name: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EsportsBannerMaterialController {}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConditionFloatClipData {
    pub m_condition_float_pair_data_list: Vec<ConditionFloatPairData>,
    pub m_child_anim_delay_switch_time: Option<f32>,
    pub updater: ConditionBoolClipDataUpdater,
    pub sync_frame_on_change_anim: Option<bool>,
    pub m_change_animation_mid_play: Option<bool>,
    pub m_play_anim_change_from_beginning: Option<bool>,
    pub dont_stomp_transition_clip: Option<bool>,
    pub m_flags: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionSpell {
    pub m_child_conditions: Vec<ContextualConditionSpellIsReady>,
    pub m_spell_slot: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    pub item_id: u32,
    pub content_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReturnToCasterOnMovementComplete {
    pub m_preserve_speed: bool,
    pub m_override_spec: AcceleratingMovement,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CursorData {
    pub m_texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnterFowVisibility {
    pub m_missile_client_wait_for_target_update_before_missile_show: Option<bool>,
    pub m_target_controls_visibility: Option<bool>,
    pub m_missile_client_exit_fow_prediction: Option<bool>,
    pub m_perception_bubble_radius: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EffectValueCalculationPart {
    pub m_effect_index: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionMoveDistance {
    pub m_distance: f32,
    pub m_compare_op: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x635d04b7 {
    pub champion_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct SkinCharacterDataProperties {
    pub alternate_icons_square: Option<Vec<String>>,
    pub health_bar_data: Option<CharacterHealthBarDataRecord>,
    pub unk_0xb67a2dd8: Option<Vec<Unk0x9c1d99c0>>,
    pub loadscreen: Option<CensoredImage>,
    pub persistent_effect_conditions: Option<Vec<MissileSpecificationBehaviors>>,
    pub m_emblems: Option<Vec<SkinEmblem>>,
    pub extra_action_button_count: Option<u32>,
    pub alternate_icons_circle: Option<Vec<String>>,
    pub unk_0x2ac577e2: Option<bool>,
    pub particle_override_death_particle: Option<String>,
    pub attribute_flags: Option<u32>,
    pub extra_character_preloads: Option<Vec<String>>,
    pub skin_classification: Option<u32>,
    pub unk_0xc3a944e7: Option<SkinCharacterDataPropertiesUnknownEnumField>,
    pub m_spawn_particle_name: Option<String>,
    pub skin_mesh_properties: Option<SkinMeshDataProperties>,
    pub unk_0xeda7817e: Option<u32>,
    pub godray_f_xbone: Option<String>,
    pub override_on_screen_name: Option<String>,
    pub icon_circle_scale: Option<f32>,
    pub uncensored_icon_squares: Option<HashMap<u32, String>>,
    pub hud_mute_event: Option<String>,
    pub particle_override_champion_kill_death_particle: Option<String>,
    pub idle_particles_effects: Option<Vec<SkinCharacterDataPropertiesCharacterIdleEffect>>,
    pub unk_0xe484edc4: Option<u32>,
    pub champion_skin_name: Option<String>,
    pub m_contextual_action_data: Option<u32>,
    pub skin_upgrade_data: Option<SkinUpgradeData>,
    pub emote_buffbone: Option<String>,
    pub m_additional_resource_resolvers: Option<Vec<u32>>,
    pub armor_material: Option<String>,
    pub secondary_resource_hud_display_data: Option<SecondaryResourceDisplayFractional>,
    pub uncensored_icon_circles: Option<HashMap<u32, String>>,
    pub skin_audio_properties: Option<SkinAudioProperties>,
    pub skin_parent: Option<i32>,
    pub icon_square: Option<String>,
    pub default_animations: Option<Vec<String>>,
    pub loadscreen_vintage: Option<CensoredImage>,
    pub can_share_theme_music: Option<bool>,
    pub theme_music: Option<Vec<String>>,
    pub icon_circle: Option<String>,
    pub skin_animation_properties: SkinAnimationProperties,
    pub icon_avatar: Option<String>,
    pub meta_data_tags: Option<String>,
    pub emote_y_offset: Option<f32>,
    pub emote_loadout: Option<u32>,
    pub m_resource_resolver: Option<u32>,
    pub hud_unmute_event: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldNoiseDefinitionData {
    pub position: Option<ValueVector3>,
    pub radius: Option<ValueFloat>,
    pub axis_fraction: Option<Vec3>,
    pub frequency: Option<ValueFloat>,
    pub velocity_delta: Option<ValueFloat>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualActionData {
    pub m_cooldown: Option<f32>,
    pub m_health_percent_threshold: Option<f32>,
    pub m_situations: HashMap<u32, ContextualSituation>,
    pub m_object_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubmeshVisibilityBoolDriver {
    pub any_submesh: Option<bool>,
    pub submeshes: Vec<u32>,
    pub visible: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionSpline {
    pub spline_width: FloatPerSpellLevel,
    pub end_locator: DrawablePositionLocator,
    pub base_texture_name: String,
    pub override_spline: HermiteSplineInfo,
    pub is_constrained_to_range: bool,
    pub constraint_range: FloatPerSpellLevel,
    pub front_texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeSphere {
    pub flags: Option<u8>,
    pub radius: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialChildTechniqueDef {
    pub shader_macros: HashMap<String, String>,
    pub name: String,
    pub parent_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BotsSpellData {
    pub unk_0xec17e271: Option<Vec<Unk0xb09016f6>>,
    pub unk_0x6d548702: Option<GameCalculation>,
    pub damage_tag: Option<u32>,
    pub unk_0x591f8423: Option<f32>,
    pub unk_0x38382c53: Option<Vec<Unk0x150d1b92>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FontType {
    pub locale_types: Vec<FontLocaleType>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxTrailDefinitionData {
    pub m_birth_tiling_size: Option<ValueVector3>,
    pub m_max_added_per_frame: Option<i32>,
    pub m_smoothing_mode: Option<u8>,
    pub m_cutoff: Option<f32>,
    pub m_mode: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConformToPathRigPoseModifierData {
    pub m_max_bone_angle: Option<f32>,
    pub activation_angle: Option<f32>,
    pub activation_distance: Option<f32>,
    pub blend_distance: Option<f32>,
    pub m_ending_joint_name: u32,
    pub m_default_mask_name: Option<u32>,
    pub m_damping_value: Option<f32>,
    pub only_activate_in_turns: Option<bool>,
    pub m_frequency: Option<f32>,
    pub m_vel_multiplier: Option<f32>,
    pub m_starting_joint_name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveBeam {
    pub m_mesh: Option<VfxMeshDefinitionData>,
    pub m_beam: Option<VfxBeamDefinitionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapParticle {
    pub name: String,
    pub system: u32,
    pub visibility_mode: Option<u32>,
    pub start_disabled: Option<bool>,
    pub m_visibility_flags: Option<u8>,
    pub group_name: Option<String>,
    pub color_modulate: Option<Vec4>,
    pub visibility_controller: Option<u32>,
    pub transitional: Option<bool>,
    pub transform: Mat4,
    pub eye_candy: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogicDriverFloatParametricUpdater {
    pub driver: BlendingSwitchMaterialDriverMDefaultValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf775806c {
    pub unk_0xd1318f26: f32,
    pub team: Option<u32>,
    pub unk_0xf908963: Vec3,
    pub skin: String,
    pub character_record: String,
    pub unk_0x651de225: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SumOfSubPartsCalculationPart {
    pub m_subparts: Box<Vec<StatBySubPartCalculationPartMSubpart>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9bc366ca {
    pub skin_id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct BarracksConfig {
    pub move_speed_increase_interval_secs: f32,
    pub move_speed_increase_max_times: i32,
    pub gold_radius: f32,
    pub minion_spawn_interval_secs: f32,
    pub initial_spawn_time_secs: f32,
    pub upgrades_before_late_game_scaling: i32,
    pub exp_radius: f32,
    pub upgrade_interval_secs: f32,
    pub move_speed_increase_initial_delay_secs: f32,
    pub move_speed_increase_increment: i32,
    pub units: Vec<BarracksMinionConfig>,
    pub wave_spawn_interval_secs: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionMarkerName {
    pub m_marker_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionObjectiveTakeByMyTeam {
    pub m_taken_objective: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinMeshDataProperties {
    pub submesh_render_order: Option<String>,
    pub brush_alpha_override: Option<f32>,
    pub emitter_submesh_avatar_to_hide: Option<String>,
    pub initial_submesh_to_hide: Option<String>,
    pub reflection_map: Option<String>,
    pub reflection_fresnel_color: Option<[u8; 4]>,
    pub skin_scale: Option<f32>,
    pub initial_submesh_avatar_to_hide: Option<String>,
    pub reflection_opacity_glancing: Option<f32>,
    pub reduced_bone_skinning: Option<bool>,
    pub uses_skin_vo: Option<bool>,
    pub skeleton: Option<String>,
    pub cast_shadows: Option<bool>,
    pub self_illumination: Option<f32>,
    pub initial_submesh_shadows_to_hide: Option<String>,
    pub enable_picking: Option<bool>,
    pub texture: Option<String>,
    pub simple_skin: Option<String>,
    pub bounding_cylinder_height: Option<f32>,
    pub reflection_opacity_direct: Option<f32>,
    pub force_draw_last: Option<bool>,
    pub bounding_cylinder_radius: Option<f32>,
    pub reflection_fresnel: Option<f32>,
    pub fresnel_color: Option<[u8; 4]>,
    pub rig_pose_modifier_data: Option<Vec<SkinMeshDataPropertiesRigPoseModifierData>>,
    pub material_controller: Option<EsportsBannerMaterialController>,
    pub fresnel: Option<f32>,
    pub gloss_texture: Option<String>,
    pub emissive_texture: Option<String>,
    pub material: Option<u32>,
    pub override_bounding_box: Option<Vec3>,
    pub bounding_sphere_radius: Option<f32>,
    pub initial_submesh_mouse_overs_to_hide: Option<String>,
    pub material_override: Option<Vec<SkinMeshDataPropertiesMaterialOverride>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SinusoidalHeightSolver {
    pub m_amplitude: f32,
    pub unk_0x827af87a: Option<f32>,
    pub m_vertical_offset: Option<f32>,
    pub m_number_of_periods: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpawningUiDefinition {
    pub max_number_of_units: i32,
    pub buff_name_filter: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapTerrainPaint {
    pub terrain_paint_texture_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x61b427f {
    pub unk_0x20941997: Option<u32>,
    pub unk_0xdba9e788: Option<bool>,
    pub unk_0x262bfa2e: Option<u32>,
    pub unk_0xaa8fd0c5: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClampSubPartsCalculationPart {
    pub m_floor: Option<f32>,
    pub m_subparts: Box<Vec<StatBySubPartCalculationPartMSubpart>>,
    pub m_ceiling: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatByCoefficientCalculationPart {
    pub m_stat: Option<u8>,
    pub m_coefficient: Option<f32>,
    pub m_stat_formula: Option<u8>,
    pub use_new_stats: Option<bool>,
    pub unk_0xa8cb9c14: Option<bool>,
    pub stat_type: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x6bbc3db6 {
    pub unk_0xda28e4c: [u8; 4],
    pub spell_objects: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x51ada002 {
    pub unk_0xf618789b: u32,
    pub unk_0xa5912d83: bool,
    pub unk_0x75d39a3b: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BarracksMinionConfig {
    pub minion_type: u8,
    pub unk_0x8a3fc6eb: u32,
    pub minion_upgrade_stats: MinionUpgradeConfig,
    pub wave_behavior: BarracksMinionConfigWaveBehavior,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StopAnimationEventData {
    pub m_end_frame: Option<f32>,
    pub m_name: Option<u32>,
    pub m_stop_animation_name: Option<u32>,
    pub m_start_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveArbitraryTrail {
    pub m_trail: Option<VfxTrailDefinitionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x150d1b92 {
    pub unk_0x717e686: Option<bool>,
    pub unk_0xe38f54f7: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KeyFrameFloatMapClipAccessoryData {
    pub key_frame_floatmap: Option<HashMap<u32, f32>>,
    pub name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialTechniqueDef {
    pub passes: Vec<StaticMaterialPassDef>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttackEvents {
    pub trigger_once_per_parent: bool,
    pub trigger_once_per_enemy_of_parent: bool,
    pub roll_for_critical_hit_result: bool,
    pub trigger_pre_attack: bool,
    pub trigger_only_once: bool,
    pub trigger_launch_attack: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GearData {
    pub m_self_only_portrait_icon: Option<String>,
    pub skin_mesh_properties: Option<SkinMeshDataProperties>,
    pub m_character_submeshes_to_hide: Option<Vec<u32>>,
    pub override_idle_effects: Option<Vec<SkinCharacterDataPropertiesCharacterIdleEffect>>,
    pub square_portrait_icon: Option<String>,
    pub m_vfx_resource_resolver: ResourceResolver,
    pub enable_override_idle_effects: Option<bool>,
    pub m_equip_animation: Option<String>,
    pub m_portrait_icon: Option<String>,
    pub m_character_submeshes_to_show: Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x72f86c81 {
    pub unk_0xdc9124b1: [u8; 4],
    pub unk_0xccfd27e6: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinFilterData {
    pub use_valid_parent_for_chroma: Option<bool>,
    pub skin_ids: Option<Vec<u32>>,
    pub filter_type: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitivePlanarProjection {
    pub m_projection: Option<VfxProjectionDefinitionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFloatOverLifeMaterialDriver {
    pub graph: VfxAnimatedFloatVariableData,
    pub frequency: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlexValueVector2 {
    pub m_value: Option<ValueVector2>,
    pub m_flex_id: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceList {
    pub level_count: Option<u32>,
    pub elements: Option<Vec<TooltipInstanceListElement>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeBox {
    pub size: Option<Vec3>,
    pub flags: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceBuff {
    pub m_object_name: Option<String>,
    pub m_loc_keys: Option<HashMap<String, String>>,
    pub m_format: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxParentInheritanceParams {
    pub mode: Option<u8>,
    pub relative_offset: Option<ValueVector3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasBuffRequirement {
    pub m_buff_name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TreatAutoAttacksAsNormalSpells {
    pub override_queryable_attack_range: GameCalculation,
    pub skip_sequenced_attack_events: bool,
    pub auto_attack_spells_use_spell_source: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FontLocaleType {
    pub m_font_file_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9be57ed9 {}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncGroupData {
    pub m_type: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParallelClipData {
    pub m_flags: Option<u32>,
    pub m_clip_name_list: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ByCharLevelBreakpointsCalculationPart {
    pub m_breakpoints: Option<Vec<Breakpoint>>,
    pub m_initial_bonus_per_level: Option<f32>,
    pub m_level1_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolSoundData {
    pub death: Option<String>,
    pub click: Option<Vec<String>>,
    pub ready: Option<String>,
    pub attack: Option<Vec<String>>,
    pub r#move: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterHealthBarDataRecord {
    pub hp_per_tick: Option<f32>,
    pub unit_health_bar_style: Option<u8>,
    pub character_state_indicator_max_count: Option<u32>,
    pub alpha_out_while_untargetable: Option<bool>,
    pub show_character_state_indicator_to_enemies: Option<bool>,
    pub show_while_untargetable: Option<bool>,
    pub attach_to_bone: Option<String>,
    pub show_character_state_indicator_to_allies: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimedWaveBehaviorInfo {
    pub behavior: BarracksMinionConfigWaveBehavior,
    pub start_time_secs: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapScriptLocator {
    pub script_name: String,
    pub name: String,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmissionSurfaceData {
    pub animation_name: Option<String>,
    pub unk_0x21af2f43: Option<u8>,
    pub mesh_scale: Option<f32>,
    pub skeleton_name: Option<String>,
    pub submeshes: Option<Vec<u32>>,
    pub use_surface_normal_for_birth_physics: Option<bool>,
    pub mesh_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameModeConstantStringVector {
    pub m_value: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AcceleratingMovement {
    pub m_project_target_to_cast_range: Option<bool>,
    pub m_start_bone_skin_overrides: Option<HashMap<u32, String>>,
    pub m_visuals_track_hidden_targets: Option<bool>,
    pub m_use_ground_height_at_target: Option<bool>,
    pub m_tracks_target: Option<bool>,
    pub m_acceleration: Option<f32>,
    pub m_min_speed: Option<f32>,
    pub m_use_height_offset_at_end: Option<bool>,
    pub m_max_speed: f32,
    pub m_initial_speed: Option<f32>,
    pub m_offset_initial_target_height: Option<f32>,
    pub m_start_bone_name: Option<String>,
    pub m_target_height_augment: Option<f32>,
    pub m_target_bone_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuffCounterDynamicMaterialFloatDriver {
    pub spell: Option<u32>,
    pub m_script_name: Option<String>,
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
pub struct Unk0x8a96ea3c {
    pub m_subparts: Box<Vec<StatBySubPartCalculationPartMSubpart>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameModeConstantsGroup {}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameModeConstantInteger {
    pub m_value: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IsSpecifiedUnitCastRequirement {
    pub m_unit: u32,
    pub m_invert_result: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct MapContainer {
    pub convert_streams_to_half_float: bool,
    pub bounds_max: Vec2,
    pub chunks: HashMap<u32, u32>,
    pub map_path: String,
    pub components: Vec<MissileSpecificationBehaviors>,
    pub lowest_walkable_height: f32,
    pub mesh_combine_radius: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxMaterialOverrideDefinitionData {
    pub material: Option<u32>,
    pub transition_sample: Option<f32>,
    pub transition_source: Option<u32>,
    pub override_blend_mode: Option<u32>,
    pub base_texture: Option<String>,
    pub priority: Option<i32>,
    pub sub_mesh_name: Option<String>,
    pub gloss_texture: Option<String>,
    pub transition_texture: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemapFloatMaterialDriver {
    pub m_max_value: Option<f32>,
    pub m_driver: Box<BlendingSwitchMaterialDriverMDefaultValue>,
    pub m_output_max_value: Option<f32>,
    pub m_min_value: Option<f32>,
    pub m_output_min_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9aa5b4bc {
    pub transform: Mat4,
    pub name: u32,
    pub definition: Unk0x7faa90a0,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x1d452085 {
    pub unk_0x137cf12a: u32,
    pub unk_0xa519b194: u32,
    pub m_stat: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xb7b43e1d {
    pub bool_driver: IsAnimationPlayingDynamicMaterialBoolDriver,
    pub percentage: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicMaterialTextureSwapDef {
    pub options: Option<Vec<DynamicMaterialTextureSwapOption>>,
    pub enabled: Option<bool>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbilityResourceDynamicMaterialFloatDriver {}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubmeshVisibilityEventData {
    pub m_name: Option<u32>,
    pub m_hide_submesh_list: Option<Vec<u32>>,
    pub m_is_self_only: Option<bool>,
    pub m_start_frame: Option<f32>,
    pub apply_only_to_avatar_meshes: Option<bool>,
    pub m_end_frame: Option<f32>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub m_show_submesh_list: Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LaunchAreaData {
    pub indicator_texture_name: String,
    pub inner_radius: f32,
    pub inner_area_target_distance: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x770f7888 {
    pub base_hp: Option<f32>,
    pub damage_per_level: Option<f32>,
    pub hp_per_level: Option<f32>,
    pub base_armor: Option<f32>,
    pub armor_per_level: Option<f32>,
    pub attack_speed_per_level: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldOrbitalDefinitionData {
    pub direction: Option<ValueVector3>,
    pub is_local_space: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetLaserComponentEffects {
    pub beam_effect_definition: SkinCharacterDataPropertiesCharacterIdleEffect,
    pub champ_targeting_effect_definition: Option<SkinCharacterDataPropertiesCharacterIdleEffect>,
    pub tower_targeting_effect_definition: Option<SkinCharacterDataPropertiesCharacterIdleEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionSpellName {
    pub m_spell: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x55f6bf86 {
    pub effect_key: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NotMaterialDriver {
    pub m_driver: Box<BlendingSwitchMaterialDriverMDefaultValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OneTrueMaterialDriver {
    pub m_drivers: Box<Option<Vec<BlendingSwitchMaterialDriverMDefaultValue>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerkReplacementList {
    pub m_replacements: Vec<PerkReplacement>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JunglePathRecommendation {
    pub order_jungle_path: Vec<JunglePathRecommendationOrderJunglePath>,
    pub chaos_jungle_path: Vec<JunglePathRecommendationOrderJunglePath>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxProjectionDefinitionData {
    pub m_y_range: Option<f32>,
    pub color_modulate: Option<ValueColor>,
    pub m_fading: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionMinimap {
    pub center_locator: Option<DrawablePositionLocator>,
    pub use_caster_bounding_box: Option<bool>,
    pub override_base_range: Option<FloatPerSpellLevel>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionAnyOtherHero {
    pub m_child_conditions: Vec<ContextualConditionCharacterMChildConditions>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxLingerDefinitionData {
    pub keyed_linger_drag: Option<ValueVector3>,
    pub keyed_linger_velocity: Option<ValueVector3>,
    pub use_linger_rotation: Option<bool>,
    pub use_keyed_linger_drag: Option<bool>,
    pub use_linger_scale: Option<bool>,
    pub use_separate_linger_color: Option<bool>,
    pub keyed_linger_acceleration: Option<ValueVector3>,
    pub use_keyed_linger_velocity: Option<bool>,
    pub separate_linger_color: Option<ValueColor>,
    pub linger_scale: Option<ValueVector3>,
    pub linger_rotation: Option<ValueVector3>,
    pub use_keyed_linger_acceleration: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParametricMovementEntry {
    pub value: Option<f32>,
    pub movement_spec: FixedSpeedSplineMovement,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwitchMaterialDriver {
    pub m_default_value: Box<BlendingSwitchMaterialDriverMDefaultValue>,
    pub m_elements: Box<Option<Vec<SwitchMaterialDriverElement>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParametricClipData {
    pub updater: ConditionBoolClipDataUpdater,
    pub m_track_data_name: u32,
    pub m_mask_data_name: Option<u32>,
    pub m_flags: Option<u32>,
    pub m_event_data_map: Option<HashMap<u32, AtomicClipDataMEventDataMap>>,
    pub unk_0x69de8fca: Option<bool>,
    pub m_parametric_pair_data_list: Vec<ParametricPairData>,
    pub m_sync_group_data_name: Option<u32>,
    pub m_animation_interruption_group_names: Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpecificColorMaterialDriver {
    pub m_color: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TerrainType {
    pub m_wall_cursor: CursorData,
    pub m_brush_cursor: CursorData,
    pub m_river_cursor: CursorData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DrawablePositionLocator {
    pub distance_offset: Option<f32>,
    pub base_position: Option<u32>,
    pub angle_offset_radian: Option<f32>,
    pub orientation_type: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualActionPlayVo {
    pub m_self_event_name: Option<String>,
    pub m_enemy_event_name: Option<String>,
    pub m_ally_event_name: Option<String>,
    pub m_spectator_event_name: Option<String>,
    pub m_wait_for_announcer_queue: Option<bool>,
    pub m_wait_timeout: Option<f32>,
    pub m_max_occurences: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xa21d6491 {
    pub unk_0xc8140e7: Vec<Unk0x11a27f9e>,
    pub frame: u32,
    pub scene: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicMaterialDef {
    pub parameters: Option<Vec<DynamicMaterialParameterDef>>,
    pub textures: Option<Vec<DynamicMaterialTextureSwapDef>>,
    pub static_switch: Option<DynamicMaterialStaticSwitch>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveMesh {
    pub m_mesh: Option<VfxMeshDefinitionData>,
    pub align_pitch_to_camera: Option<bool>,
    pub unk_0x6aec9e7a: Option<bool>,
    pub align_yaw_to_camera: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MissileGroupSpawnerSpec {
    pub m_child_missile_spell: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicMaterialParameterDef {
    pub enabled: Option<bool>,
    pub driver: BlendingSwitchMaterialDriverMDefaultValue,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xe7b61183 {
    pub unk_0x44146c9d: Vec<u32>,
    pub unk_0x8f149e18: f32,
    pub unk_0xe1795243: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xec733fe2 {
    pub default_visible: Option<bool>,
    pub unk_0x8bff8cdf: u8,
    pub name: u32,
    pub path_hash: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FadeOverTimeBehavior {
    pub m_time_end: f32,
    pub m_start_alpha: Option<f32>,
    pub m_end_alpha: Option<f32>,
    pub m_time_start: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdaterData {
    pub input: ConditionBoolClipDataUpdater,
    pub m_output_type: u32,
    pub m_value_processor_data_list: Option<Vec<LinearTransformProcessorData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FloatTextFormattingData {
    pub min_y_velocity: f32,
    pub ignore_queue: Option<bool>,
    pub disable_vertical_reverse: Option<bool>,
    pub hang_time: Option<f32>,
    pub shrink_scale: Option<f32>,
    pub follow_source: Option<bool>,
    pub shrink_time: Option<f32>,
    pub continual_force_y: Option<f32>,
    pub scale: Option<f32>,
    pub start_offset_y: f32,
    pub icons: Vec<FloatTextIconData>,
    pub extend_time_on_new_damage: Option<f32>,
    pub min_x_velocity: f32,
    pub max_y_velocity: f32,
    pub decay: f32,
    pub combinable_counter_display: Option<bool>,
    pub combinable_counter_category: Option<i32>,
    pub decay_delay: f32,
    pub combinable_number_format: Option<String>,
    pub max_x_velocity: f32,
    pub max_life_time: Option<f32>,
    pub start_offset_x: Option<f32>,
    pub m_type_name: u32,
    pub priority: i32,
    pub is_animated: bool,
    pub m_font_description: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnimationResourceData {
    pub m_animation_file_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xad65d8c4 {
    pub definition: Unk0x9d9f60d2,
    pub name: u32,
    pub transform: Mat4,
    pub unk_0xbbe68da1: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionCharacterPlayingAnimation {
    pub m_animation_name_hash: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MaxMaterialDriver {
    pub m_drivers: Box<Vec<BlendingSwitchMaterialDriverMDefaultValue>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapBakeProperties {
    pub light_grid_size: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxMaterialDefinitionData {
    pub material: u32,
    pub material_drivers: Option<HashMap<String, VfxMaterialDefinitionDataMaterialDrivers>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9d62f7e {
    pub named_data_value: u32,
    pub spell: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimeBlendData {
    pub m_time: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x4ab36eb5 {
    pub unk_0x93f0c42c: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UseableData {
    pub use_spell_name: Option<String>,
    pub use_hero_spell_name: Option<String>,
    pub use_cooldown_spell_slot: Option<i32>,
    pub flags: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedSpeedMovement {
    pub m_speed: Option<f32>,
    pub m_use_height_offset_at_end: Option<bool>,
    pub m_target_height_augment: Option<f32>,
    pub m_start_bone_name: Option<String>,
    pub m_target_bone_name: Option<String>,
    pub m_use_ground_height_at_target: Option<bool>,
    pub m_project_target_to_cast_range: Option<bool>,
    pub m_infer_direction_from_facing_if_needed: Option<bool>,
    pub unk_0x3046674: Option<bool>,
    pub m_visuals_track_hidden_targets: Option<bool>,
    pub add_bonus_attack_range_to_cast_range: Option<bool>,
    pub m_offset_initial_target_height: Option<f32>,
    pub m_start_bone_skin_overrides: Option<HashMap<u32, String>>,
    pub m_tracks_target: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct MapPlaceableContainer {
    pub items: HashMap<u32, MissileSpecificationBehaviors>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MaskData {
    pub m_weight_list: Vec<f32>,
    pub mid: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FloatGraphMaterialDriver {
    pub graph: VfxAnimatedFloatVariableData,
    pub driver: Box<BlendingSwitchMaterialDriverMDefaultValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x7f796784 {}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameModeConstantString {
    pub m_value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapNavGridOverlays {
    pub overlays: Vec<MapNavGridOverlay>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PersistentVfxData {
    pub scale: Option<f32>,
    pub target_pos_is_owner: Option<bool>,
    pub bone_name: Option<String>,
    pub attach_to_camera: Option<bool>,
    pub use_different_key_for_other_team: Option<bool>,
    pub effect_key: u32,
    pub play_speed_modifier: Option<f32>,
    pub specific_team: Option<u32>,
    pub target_bone_name: Option<String>,
    pub effect_key_for_other_team: Option<u32>,
    pub show_to_owner_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GdsMapObject {
    pub r#type: u8,
    pub box_max: Option<Vec3>,
    pub extra_info: Option<Vec<GdsMapObjectBannerInfo>>,
    pub transform: Mat4,
    pub box_min: Option<Vec3>,
    pub visibility_controller: Option<u32>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SoundEventData {
    pub m_skip_if_past_end_frame: Option<bool>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub m_is_kill_event: Option<bool>,
    pub m_is_loop: Option<bool>,
    pub condition_clip_transition_type: Option<u16>,
    pub m_start_frame: Option<f32>,
    pub m_name: Option<u32>,
    pub m_sound_name: Option<String>,
    pub m_is_self_only: Option<bool>,
    pub m_end_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinAugmentCategories {
    pub basic_augments: Option<Vec<Unk0xe1555e0a>>,
    pub border_augments: Option<Vec<Unk0x4a70b12c>>,
    pub unk_0x25c5104d: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapNavGridOverlay {
    pub regions_filename: String,
    pub name: u32,
    pub nav_grid_file_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuffData {
    pub m_show_accumulated_duration: Option<bool>,
    pub can_timeout_while_casting: Option<bool>,
    pub m_buff_attribute_flag: Option<u8>,
    pub m_tooltip_data: Option<TooltipInstanceBuff>,
    pub persistent_effect_conditions: Option<Vec<PersistentEffectConditionData>>,
    pub m_description: Option<String>,
    pub m_show_duration: Option<bool>,
    pub m_float_vars_decimals: Option<Vec<i32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GravityHeightSolver {
    pub unk_0x922c17e5: Option<f32>,
    pub m_gravity: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemRecommendationOverrideStartingItemBundle {
    pub items: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DelayStart {
    pub m_delay_time: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterPassiveData {
    pub m_allow_on_clones: Option<bool>,
    pub m_parent_passive_buff: u32,
    pub m_child_spells: Option<Vec<u32>>,
    pub m_component_buffs: Option<Vec<u32>>,
    pub m_display_flags: Option<u8>,
    pub skin_filter: Option<SkinFilterData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NavGridTerrainConfig {
    pub tags: Vec<Unk0xd82714cc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuffCounterByCoefficientCalculationPart {
    pub m_buff_name: u32,
    pub m_scaling_tag_key: Option<String>,
    pub m_icon_key: Option<String>,
    pub m_coefficient: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FloatTextIconData {
    pub m_offset: Vec2,
    pub m_icon_file_name: String,
    pub m_color: Option<[u8; 4]>,
    pub m_display_size: Vec2,
    pub m_alignment: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxSoftParticleDefinitionData {
    pub begin_in: Option<f32>,
    pub unk_0x3bf176bc: Option<u8>,
    pub begin_out: Option<f32>,
    pub delta_in: Option<f32>,
    pub delta_out: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionItemId {
    pub m_items: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionAoeScalar {
    pub scalar: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionMapRegionName {
    pub m_region_type: u8,
    pub m_region_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9c5b78dd {
    pub unk_0xb4222185: Vec<u32>,
    pub resource_resolver: Unk0x20194a16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x7a1a2d27 {
    pub default_magical_damage_format: u32,
    pub critical_true_damage_format: u32,
    pub absorbed_damage_format: u32,
    pub critical_physical_damage_format: u32,
    pub combinable_damage_format: u32,
    pub critical_magical_damage_format: u32,
    pub default_physical_damage_format: u32,
    pub default_true_damage_format: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x56bb851 {
    pub unk_0xe6d60f41: Option<HashMap<u8, Unk0xc76c1b9a>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialPassDef {
    pub shader_macros: Option<HashMap<String, String>>,
    pub write_mask: Option<u32>,
    pub depth_offset_slope: Option<f32>,
    pub depth_compare_func: Option<u32>,
    pub polygon_depth_bias_enable: Option<bool>,
    pub src_alpha_blend_factor: Option<u32>,
    pub depth_enable: Option<bool>,
    pub winding_to_cull: Option<u32>,
    pub dst_alpha_blend_factor: Option<u32>,
    pub blend_enable: Option<bool>,
    pub src_color_blend_factor: Option<u32>,
    pub depth_offset_bias: Option<f32>,
    pub shader: u32,
    pub cull_enable: Option<bool>,
    pub dst_color_blend_factor: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TriggerOnHit {
    pub m_actions: Vec<TriggerOnDelayMActions>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xfcb92181 {
    pub team: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xc96d9140 {
    pub unk_0xa2cb8e03: Option<HashMap<String, u32>>,
    pub unk_0xc19c58be: Option<HashMap<String, String>>,
    pub unk_0x1418c47f: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x6355dd6f {
    pub chunk: u32,
    pub visibility_controller: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinCharacterMetaDataProperties {
    pub e_sport_character: Option<bool>,
    pub spawning_skin_offsets: Option<Vec<SkinCharacterMetaDataPropertiesSpawningSkinOffset>>,
    pub e_sport_team_table: Option<Vec<ESportTeamEntry>>,
    pub e_sport_league_table: Option<Vec<ESportLeagueEntry>>,
    pub relative_color_swap_table: Option<Vec<i32>>,
    pub skin_based_relative_color_scheme: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf6f4bb5f {
    pub name: String,
    pub color: Option<[u8; 4]>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicMaterialTextureSwapOption {
    pub texture_name: Option<String>,
    pub driver: BlendingSwitchMaterialDriverMDefaultValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WidthPerSecond {
    pub m_width_per_second: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SequencerClipData {
    pub m_clip_name_list: Vec<u32>,
    pub m_flags: Option<u32>,
    pub m_event_data_map: Option<HashMap<u32, AtomicClipDataMEventDataMap>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x7084628f {
    pub unk_0xc742ceb4: u32,
    pub custom_announcement_style: u32,
    pub unk_0x9d6e31fd: u32,
    pub bottom_hr_momentum_post: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct ResourceResolver {
    pub resource_map: Option<HashMap<u32, u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Breakpoint {
    pub m_level: Option<u32>,
    pub m_additional_bonus_at_this_level: Option<f32>,
    pub m_bonus_per_level_at_and_after: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9c1d99c0 {
    pub spells: Vec<u32>,
    pub unk_0x80cf3335: Unk0x7a1a2d27,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionGameTimer {
    pub m_compare_op: u8,
    pub m_game_time_in_minutes: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValueColor {
    pub dynamics: Option<VfxAnimatedColorVariableData>,
    pub constant_value: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeLegacy {
    pub emit_rotation_axes: Option<Vec<Vec3>>,
    pub emit_offset: Option<ValueVector3>,
    pub emit_rotation_angles: Option<Vec<ValueFloat>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualRule {
    pub stomp_lower_priority: Option<bool>,
    pub chance_weight: Option<f32>,
    pub m_rule_name: Option<String>,
    pub m_priority: Option<i32>,
    pub m_audio_action: Option<ContextualActionPlayVo>,
    pub m_conditions: Option<Vec<ContextualRuleMConditions>>,
    pub m_animation_action: Option<ContextualActionPlayAnimation>,
    pub cooldown_modifications: Option<ContextualActionCooldownModifications>,
    pub unk_0x20749c51: Option<bool>,
    pub m_override_cac_cooldown: Option<bool>,
    pub m_trigger_event_action: Option<ContextualActionTriggerEvent>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxMeshDefinitionData {
    pub m_mesh_skeleton_name: Option<String>,
    pub m_mesh_name: Option<String>,
    pub m_submeshes_to_draw_always: Option<Vec<u32>>,
    pub m_lock_mesh_to_attachment: Option<bool>,
    pub m_animation_variants: Option<Vec<String>>,
    pub m_submeshes_to_draw: Option<Vec<u32>>,
    pub m_animation_name: Option<String>,
    pub m_simple_mesh_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveCameraTrail {
    pub m_trail: Option<VfxTrailDefinitionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PersistentEffectConditionData {
    pub submeshes_to_hide: Option<Vec<u32>>,
    pub persistent_vfxs: Option<Vec<PersistentVfxData>>,
    pub owner_condition: Option<BlendingSwitchMaterialDriverMDefaultValue>,
    pub submeshes_to_show: Option<Vec<u32>>,
    pub force_render_vfx: Option<bool>,
    pub source_condition: Option<BlendingSwitchMaterialDriverMDefaultValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpringPhysicsRigPoseModifierData {
    pub default_on: Option<bool>,
    pub name: Option<u32>,
    pub damping: Option<f32>,
    pub joint: u32,
    pub max_angle: Option<f32>,
    pub max_distance: Option<f32>,
    pub do_translation: Option<bool>,
    pub do_rotation: bool,
    pub spring_stiffness: Option<f32>,
    pub mass: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextualConditionNegation {
    pub m_child_condition: Box<ContextualRuleMConditions>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x1f2e5fd0 {
    pub unk_0x752ff961: u32,
    pub group: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedDurationTriggeredBoolDriver {
    pub m_custom_duration: f32,
    pub m_bool_driver: Box<BlendingSwitchMaterialDriverMDefaultValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf3cbe7b2 {
    pub m_spell_calculation_key: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xd82714cc {
    pub name: u32,
    pub color: Option<[u8; 4]>,
    pub flags: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatByNamedDataValueCalculationPart {
    pub m_data_value: u32,
    pub m_stat_formula: Option<u8>,
    pub m_stat: Option<u8>,
    pub use_new_stats: Option<bool>,
    pub stat_type: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemRecommendationOverrideSet {
    pub m_overrides: Vec<ItemRecommendationOverride>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttackSlotData {
    pub m_attack_total_time: Option<f32>,
    pub m_attack_name: Option<String>,
    pub m_attack_delay_cast_offset_percent: Option<f32>,
    pub m_attack_cast_time: Option<f32>,
    pub m_attack_delay_cast_offset_percent_attack_speed_ratio: Option<f32>,
    pub m_override_autoattack_cast_time: Option<OverrideAutoAttackCastTimeData>,
    pub m_attack_probability: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FloatComparisonMaterialDriver {
    pub m_value_a: Box<BlendingSwitchMaterialDriverMDefaultValue>,
    pub m_operator: Option<u32>,
    pub m_value_b: Box<BlendingSwitchMaterialDriverMDefaultValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldCollectionDefinitionData {
    pub field_attraction_definitions: Option<Vec<VfxFieldAttractionDefinitionData>>,
    pub field_orbital_definitions: Option<Vec<VfxFieldOrbitalDefinitionData>>,
    pub field_noise_definitions: Option<Vec<VfxFieldNoiseDefinitionData>>,
    pub field_acceleration_definitions: Option<Vec<VfxFieldAccelerationDefinitionData>>,
    pub field_drag_definitions: Option<Vec<VfxFieldDragDefinitionData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BorderSkinAugment {
    pub m_name_tra_key: String,
    pub catalog_entry: CatalogEntry,
    pub border: BorderPropertyData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrackData {
    pub m_priority: Option<u8>,
    pub m_blend_weight: Option<f32>,
    pub m_blend_mode: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecSpellRankUpInfo {
    pub m_default_priority: Option<Vec<u32>>,
    pub mode_name_string_id: Option<u32>,
    pub map_id: Option<u32>,
    pub m_early_level_overrides: Option<Vec<u32>>,
    pub is_default_recommendation: Option<bool>,
    pub unk_0x5b968ffb: Option<bool>,
    pub position: Option<u32>,
}
