use std::collections::HashMap;

use bevy::animation::graph::AnimationNodeIndex;
use league_core::extract::{
    AnimationGraphData, AtomicClipData, ConditionBoolClipData, ConditionFloatClipData,
    EnumBlendData, EnumClipData, EnumParametricUpdater, ParallelClipData, ParametricClipData,
    SelectorClipData, SequencerClipData,
};
use lol_base::animation::{
    ConfigAnimationNode, ConfigAnimationNodeF32, ConfigBlendData, ConfigParametricUpdater,
    LOLAnimationGraph,
};

/// Convert EnumParametricUpdater to ConfigParametricUpdater
pub fn parametric_updater_to_config(updater: &EnumParametricUpdater) -> ConfigParametricUpdater {
    match updater {
        EnumParametricUpdater::AttackSpeedParametricUpdater => ConfigParametricUpdater::AttackSpeed,
        EnumParametricUpdater::DisplacementParametricUpdater => {
            ConfigParametricUpdater::Displacement
        }
        EnumParametricUpdater::EquippedGearParametricUpdater => {
            ConfigParametricUpdater::EquippedGear
        }
        EnumParametricUpdater::FacingAndMovementAngleParametricUpdater => {
            ConfigParametricUpdater::FacingAndMovementAngle
        }
        EnumParametricUpdater::FacingParametricUpdater => ConfigParametricUpdater::Facing,
        EnumParametricUpdater::IsAllyParametricUpdater => ConfigParametricUpdater::IsAlly,
        EnumParametricUpdater::IsHomeguardParametricUpdater => ConfigParametricUpdater::IsHomeguard,
        EnumParametricUpdater::IsInTerrainParametricUpdater => ConfigParametricUpdater::IsInTerrain,
        EnumParametricUpdater::IsMovingParametricUpdater => ConfigParametricUpdater::IsMoving,
        EnumParametricUpdater::IsRangedParametricUpdater => ConfigParametricUpdater::IsRanged,
        EnumParametricUpdater::IsTurningParametricUpdater => ConfigParametricUpdater::IsTurning,
        EnumParametricUpdater::LogicDriverBoolParametricUpdater(_) => {
            ConfigParametricUpdater::LogicDriverBool
        }
        EnumParametricUpdater::LogicDriverFloatParametricUpdater(_) => {
            ConfigParametricUpdater::LogicDriverFloat
        }
        EnumParametricUpdater::LookAtGoldRedirectTargetAngleParametricUpdater => {
            ConfigParametricUpdater::LookAtGoldRedirectTargetAngle
        }
        EnumParametricUpdater::LookAtInterestAngleParametricUpdater => {
            ConfigParametricUpdater::LookAtInterestAngle
        }
        EnumParametricUpdater::LookAtInterestDistanceParametricUpdater => {
            ConfigParametricUpdater::LookAtInterestDistance
        }
        EnumParametricUpdater::LookAtSpellTargetAngleParametricUpdater => {
            ConfigParametricUpdater::LookAtSpellTargetAngle
        }
        EnumParametricUpdater::LookAtSpellTargetDistanceParametricUpdater => {
            ConfigParametricUpdater::LookAtSpellTargetDistance
        }
        EnumParametricUpdater::LookAtSpellTargetHeightOffsetParametricUpdater => {
            ConfigParametricUpdater::LookAtSpellTargetHeightOffset
        }
        EnumParametricUpdater::MoveSpeedParametricUpdater => ConfigParametricUpdater::MoveSpeed,
        EnumParametricUpdater::MovementDirectionParametricUpdater => {
            ConfigParametricUpdater::MovementDirection
        }
        EnumParametricUpdater::ParBarPercentParametricUpdater => {
            ConfigParametricUpdater::ParBarPercent
        }
        EnumParametricUpdater::SkinScaleParametricUpdater => ConfigParametricUpdater::SkinScale,
        EnumParametricUpdater::SlopeAngleParametricUpdater => ConfigParametricUpdater::SlopeAngle,
        EnumParametricUpdater::TotalTurnAngleParametricUpdater => {
            ConfigParametricUpdater::TotalTurnAngle
        }
        EnumParametricUpdater::TurnAngleParametricUpdater => ConfigParametricUpdater::TurnAngle,
        EnumParametricUpdater::TurnAngleRemainingParametricUpdater => {
            ConfigParametricUpdater::TurnAngleRemaining
        }
        _ => ConfigParametricUpdater::Unknown,
    }
}

/// Convert EnumBlendData to ConfigBlendData
pub fn blend_data_to_config(
    blend_data: &EnumBlendData,
    hashes: &HashMap<u32, String>,
) -> ConfigBlendData {
    match blend_data {
        EnumBlendData::TimeBlendData(time_blend) => ConfigBlendData::Time {
            time: time_blend.m_time.unwrap_or(0.0),
        },
        EnumBlendData::TransitionClipBlendData(transition_clip) => {
            ConfigBlendData::TransitionClip {
                clip_name: league_utils::hash_to_type_name(&transition_clip.m_clip_name, hashes),
            }
        }
    }
}

/// Convert EnumClipData to ConfigAnimationNode
pub fn clip_data_to_node(
    _hash: u32,
    clip: &EnumClipData,
    hashes: &HashMap<u32, String>,
) -> ConfigAnimationNode {
    match clip {
        EnumClipData::AtomicClipData(AtomicClipData {
            m_animation_resource_data: _,
            ..
        }) => {
            // The node_index is assigned later when building the AnimationGraph
            // For now, we return a placeholder that will be resolved later
            ConfigAnimationNode::Clip {
                node_index: AnimationNodeIndex::new(0), // Will be updated by build_animation_nodes
            }
        }
        EnumClipData::ConditionFloatClipData(ConditionFloatClipData {
            m_condition_float_pair_data_list,
            updater,
            ..
        }) => ConfigAnimationNode::ConditionFloat {
            conditions: m_condition_float_pair_data_list
                .iter()
                .map(|v| ConfigAnimationNodeF32 {
                    key: league_utils::hash_to_type_name(&v.m_clip_name, hashes),
                    value: v.m_value.unwrap_or(0.0),
                })
                .collect(),
            updater: parametric_updater_to_config(updater),
        },
        EnumClipData::SelectorClipData(SelectorClipData {
            m_selector_pair_data_list,
            ..
        }) => ConfigAnimationNode::Selector {
            probably_nodes: m_selector_pair_data_list
                .iter()
                .map(|v| ConfigAnimationNodeF32 {
                    key: league_utils::hash_to_type_name(&v.m_clip_name, hashes),
                    value: v.m_probability.unwrap_or(0.0),
                })
                .collect(),
        },
        EnumClipData::SequencerClipData(SequencerClipData {
            m_clip_name_list, ..
        }) => ConfigAnimationNode::Sequence {
            hashes: m_clip_name_list
                .iter()
                .map(|h| league_utils::hash_to_type_name(h, hashes))
                .collect(),
        },
        EnumClipData::ParallelClipData(ParallelClipData {
            m_clip_name_list, ..
        }) => ConfigAnimationNode::Parallel {
            hashes: m_clip_name_list
                .iter()
                .map(|h| league_utils::hash_to_type_name(h, hashes))
                .collect(),
        },
        EnumClipData::ParametricClipData(ParametricClipData {
            m_parametric_pair_data_list,
            ..
        }) => ConfigAnimationNode::Parametric {
            pairs: m_parametric_pair_data_list
                .iter()
                .map(|v| ConfigAnimationNodeF32 {
                    key: league_utils::hash_to_type_name(&v.m_clip_name, hashes),
                    value: v.m_value.unwrap_or(0.0),
                })
                .collect(),
        },
        EnumClipData::ConditionBoolClipData(ConditionBoolClipData {
            updater,
            m_true_condition_clip_name,
            m_false_condition_clip_name,
            ..
        }) => ConfigAnimationNode::ConditionBool {
            updater: parametric_updater_to_config(updater),
            true_node: league_utils::hash_to_type_name(m_true_condition_clip_name, hashes),
            false_node: league_utils::hash_to_type_name(m_false_condition_clip_name, hashes),
        },
    }
}

/// Convert AnimationGraphData to ConfigAnimation
pub fn animation_graph_to_config(
    graph_data: &AnimationGraphData,
    node_index_map: &HashMap<u32, AnimationNodeIndex>,
    hashes: &HashMap<u32, String>,
    gltf_path: String,
) -> LOLAnimationGraph {
    let mut hash_to_node = HashMap::new();

    // Convert clip data map to nodes
    if let Some(clip_data_map) = &graph_data.m_clip_data_map {
        for (hash, clip) in clip_data_map {
            let mut node = clip_data_to_node(*hash, clip, hashes);
            // Update Clip nodes with correct node_index
            if let ConfigAnimationNode::Clip { node_index } = &mut node {
                *node_index = *node_index_map.get(hash).unwrap_or(node_index);
            }
            // Convert hash to name for the key
            let name = league_utils::hash_to_type_name(hash, hashes);
            hash_to_node.insert(name, node);
        }
    }

    // Add Attack selector node (same as in lol_render)
    hash_to_node.insert(
        "Attack".to_string(),
        ConfigAnimationNode::Selector {
            probably_nodes: vec![
                ConfigAnimationNodeF32 {
                    key: "Attack1".to_string(),
                    value: 1.0,
                },
                ConfigAnimationNodeF32 {
                    key: "Attack2".to_string(),
                    value: 1.0,
                },
            ],
        },
    );

    // Convert blend data
    let mut blend_data = HashMap::new();
    if let Some(blend_table) = &graph_data.m_blend_data_table {
        for (key, value) in blend_table {
            // Key is u64, split into two u32
            let high = (key >> 32) as u32;
            let low = (key & 0xFFFFFFFF) as u32;
            let high_name = league_utils::hash_to_type_name(&high, hashes);
            let low_name = league_utils::hash_to_type_name(&low, hashes);
            blend_data.insert((high_name, low_name), blend_data_to_config(value, hashes));
        }
    }

    LOLAnimationGraph {
        gltf_path,
        hash_to_node,
        blend_data,
    }
}
