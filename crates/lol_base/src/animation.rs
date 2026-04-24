use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Animation clip data - stores keyframe animation data
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConfigAnimationClip {
    pub fps: f32,
    pub duration: f32,
    pub joint_hashes: Vec<u32>,
    pub translates: Vec<Vec<(f32, Vec3)>>,
    pub rotations: Vec<Vec<(f32, Quat)>>,
    pub scales: Vec<Vec<(f32, Vec3)>>,
}

/// Animation node index type
pub type AnimationNodeIndex = usize;

/// Animation graph asset - stable version of lol_render::Animation
#[derive(Asset, TypePath, Clone, Serialize, Deserialize)]
pub struct ConfigAnimation {
    pub hash_to_node: HashMap<u32, ConfigAnimationNode>,
    pub blend_data: HashMap<(u32, u32), ConfigBlendData>,
}

/// Animation node types
#[derive(Clone, Serialize, Deserialize)]
pub enum ConfigAnimationNode {
    Clip {
        node_index: AnimationNodeIndex,
    },
    ConditionFloat {
        updater: ConfigParametricUpdater,
        conditions: Vec<ConfigAnimationNodeF32>,
    },
    Selector {
        probably_nodes: Vec<ConfigAnimationNodeF32>,
    },
    Sequence {
        hashes: Vec<u32>,
    },
    ConditionBool {
        updater: ConfigParametricUpdater,
        true_node: u32,
        false_node: u32,
    },
}

/// Float condition for animation nodes
#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigAnimationNodeF32 {
    pub key: u32,
    pub value: f32,
}

/// Parametric updater types - stable version
#[derive(Clone, Serialize, Deserialize)]
pub enum ConfigParametricUpdater {
    AttackSpeed,
    Displacement,
    EquippedGear,
    FacingAndMovementAngle,
    Facing,
    IsAlly,
    IsHomeguard,
    IsInTerrain,
    IsMoving,
    IsRanged,
    IsTurning,
    LogicDriverBool,
    LogicDriverFloat,
    LookAtGoldRedirectTargetAngle,
    LookAtInterestAngle,
    LookAtInterestDistance,
    LookAtSpellTargetAngle,
    LookAtSpellTargetDistance,
    LookAtSpellTargetHeightOffset,
    MoveSpeed,
    MovementDirection,
    ParBarPercent,
    SkinScale,
    SlopeAngle,
    TotalTurnAngle,
    TurnAngle,
    TurnAngleRemaining,
    Unknown,
}

/// Blend data types - stable version
#[derive(Clone, Serialize, Deserialize)]
pub enum ConfigBlendData {
    Time { time: f32 },
    TransitionClip { clip_name: u32 },
}

/// Animation handler component - holds handle to ConfigAnimation asset
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct AnimationHandler(pub Handle<ConfigAnimation>);
