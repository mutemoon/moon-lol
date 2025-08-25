use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::league::BinHash;

// AnimationGraphData 的主结构体
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnimationGraphData {
    pub m_cascade_blend_value: f32,
    pub m_clip_data_map: HashMap<u32, ClipData>,
    pub m_track_data_map: HashMap<BinHash, TrackData>,
    pub m_blend_data_table: HashMap<u64, BlendData>,
}

// ClipData 的枚举类型，包含不同的剪辑数据类型
#[derive(Serialize, Deserialize, Debug)]
pub enum ClipData {
    AtomicClipData(AtomicClipData),
    BlendableClipData,
    ConditionBoolClipData,
    ConditionFloatClipData(ConditionFloatClipData),
    EventControlledSelectorClipData,
    ParallelClipData,
    ParametricClipData(ParametricClipData),
    SelectorClipData(SelectorClipData),
    SequencerClipData,
    StateAnimClipData,
    SwitchIntClipData,
    TransitionClipData,
}

// ParametricClipData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParametricClipData {
    pub m_track_data_name: BinHash,
    pub m_event_data_map: Option<HashMap<BinHash, EventData>>,
    pub updater: AnimationConditionUpdater,
    pub m_parametric_pair_data_list: Vec<ParametricPairData>,
}

// ParametricPairData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParametricPairData {
    pub m_clip_name: BinHash,
    #[serde(default)]
    pub m_value: f32,
}

// AtomicClipData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AtomicClipData {
    pub m_flags: Option<u32>,
    pub m_track_data_name: BinHash,
    pub m_tick_duration: Option<f32>,
    pub m_event_data_map: Option<HashMap<BinHash, EventData>>,
    pub m_animation_resource_data: AnimationResourceData,
}

// SelectorClipData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SelectorClipData {
    pub m_flags: Option<u32>,
    pub m_selector_pair_data_list: Vec<SelectorPairData>,
}

// ConditionFloatClipData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConditionFloatClipData {
    pub m_flags: Option<u32>,
    pub m_condition_float_pair_data_list: Vec<ConditionFloatPairData>,
    pub updater: AnimationConditionUpdater,
    pub m_change_animation_mid_play: bool,
}

// EventData 的枚举类型
#[derive(Serialize, Deserialize, Debug)]
pub enum EventData {
    ParticleEventData(ParticleEventData),
    SoundEventData(SoundEventData),
    JointSnapEventData(JointSnapEventData),
    FaceTargetEventData(FaceTargetEventData),
    SubmeshVisibilityEventData(SubmeshVisibilityEventData),
    EnableLookAtEventData(EnableLookAtEventData),
    IdleParticlesVisibilityEventData,
    StopAnimationEventData,
    LockRootOrientationEventData,
    SpringPhysicsEventData,
    JointOrientationEventData,
    SyncedAnimationEventData,
    ConformToPathEventData,
}

// EnableLookAtEventData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnableLookAtEventData {
    pub m_name: BinHash,
    pub m_start_frame: Option<f32>,
    pub m_end_frame: f32,
    #[serde(default = "default_true")]
    pub m_enable_look_at: bool,
    #[serde(default = "default_true")]
    pub m_lock_current_values: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEventData {
    pub m_name: Option<BinHash>,
    pub m_start_frame: Option<f32>,
    pub m_effect_key: Option<BinHash>,
    pub m_particle_event_data_pair_list: Vec<ParticleEventDataPair>,
    #[serde(default = "default_true")]
    pub m_is_loop: bool,
    pub m_is_kill_event: Option<bool>,
}

// SoundEventData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SoundEventData {
    pub m_start_frame: Option<f32>,
    pub m_sound_name: Option<String>,
    #[serde(default = "default_true")]
    pub m_is_loop: bool,
}

// JointSnapEventData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JointSnapEventData {
    pub m_start_frame: Option<f32>,
    pub m_end_frame: Option<f32>,
    pub m_name: Option<BinHash>,
    pub m_joint_name_to_override: BinHash,
    pub m_joint_name_to_snap_to: BinHash,
}

// FaceTargetEventData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FaceTargetEventData {}

// SubmeshVisibilityEventData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SubmeshVisibilityEventData {
    pub m_start_frame: Option<f32>,
    pub m_hide_submesh_list: Vec<BinHash>,
}

// AnimationResourceData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnimationResourceData {
    pub m_animation_file_path: String,
}

// SelectorPairData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SelectorPairData {
    pub m_clip_name: BinHash,
    pub m_probability: f32,
}

// ConditionFloatPairData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConditionFloatPairData {
    pub m_clip_name: BinHash,
    pub m_value: Option<f32>,
}

// ParticleEventDataPair
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEventDataPair {
    pub m_bone_name: Option<BinHash>,
}

// TrackData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TrackData {}

// BlendData 的枚举类型
#[derive(Serialize, Deserialize, Debug)]
pub enum BlendData {
    TimeBlendData(TimeBlendData),
    TransitionClipBlendData(TransitionClipBlendData),
}

// TimeBlendData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimeBlendData {
    pub m_time: f32,
}

// TransitionClipBlendData
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransitionClipBlendData {
    pub m_clip_name: BinHash,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AnimationConditionUpdater {
    ArenaOwnerLevelParametricUpdater,
    AttackSpeedParametricUpdater,
    DisplacementParametricUpdater,
    EquippedGearParametricUpdater,
    FacingAndMovementAngleParametricUpdater,
    FacingParametricUpdater,
    GrassSwayDirectionParametricUpdater,
    GrassSwayIntensityParametricUpdater,
    IBaseParametricUpdater,
    IBooleanParametricUpdater,
    IFloatParametricUpdater,
    IsAllyParametricUpdater,
    IsHomeguardParametricUpdater,
    IsInTerrainParametricUpdater,
    IsMovingParametricUpdater,
    IsRangedParametricUpdater,
    IsTurningParametricUpdater,
    LogicDriverBoolParametricUpdater,
    LogicDriverFloatParametricUpdater,
    LookAtGoldRedirectTargetAngleParametricUpdater,
    LookAtInterestAngleParametricUpdater,
    LookAtInterestDistanceParametricUpdater,
    LookAtSpellTargetAngleParametricUpdater,
    LookAtSpellTargetDistanceParametricUpdater,
    LookAtSpellTargetHeightOffsetParametricUpdater,
    MoveSpeedParametricUpdater,
    MovementDirectionParametricUpdater,
    MovingTowardEnemyParametricUpdater,
    ParBarPercentParametricUpdater,
    PathingAngleParametricUpdater,
    SkinScaleParametricUpdater,
    SlopeAngleParametricUpdater,
    TftArenaOwnerLevelParametricUpdater,
    TftArenaOwnerStreakParametricUpdater,
    TotalTurnAngleParametricUpdater,
    TurnAngleParametricUpdater,
    TurnAngleRemainingParametricUpdater,
}

fn default_true() -> bool {
    true
}
