use std::collections::HashMap;

use bevy::animation::graph::AnimationNodeIndex;
use league_core::extract::{
    AnimationGraphData, AtomicClipData, ConditionBoolClipData, ConditionFloatClipData,
    EnumBlendData, EnumClipData, EnumParametricUpdater, SelectorClipData, SequencerClipData,
};
use league_utils::hash_bin;
use lol_base::animation::{
    ConfigAnimation, ConfigAnimationNode, ConfigAnimationNodeF32, ConfigBlendData,
    ConfigParametricUpdater,
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
pub fn blend_data_to_config(blend_data: &EnumBlendData) -> ConfigBlendData {
    match blend_data {
        EnumBlendData::TimeBlendData(time_blend) => ConfigBlendData::Time {
            time: time_blend.m_time.unwrap_or(0.0),
        },
        EnumBlendData::TransitionClipBlendData(transition_clip) => {
            ConfigBlendData::TransitionClip {
                clip_name: transition_clip.m_clip_name,
            }
        }
    }
}

/// Convert EnumClipData to ConfigAnimationNode
/// Returns the node and the node_index for Clip nodes
pub fn clip_data_to_node(
    _hash: u32,
    clip: &EnumClipData,
) -> (ConfigAnimationNode, Option<AnimationNodeIndex>) {
    match clip {
        EnumClipData::AtomicClipData(AtomicClipData {
            m_animation_resource_data: _,
            ..
        }) => {
            // The node_index is assigned later when building the AnimationGraph
            // For now, we return a placeholder that will be resolved later
            (
                ConfigAnimationNode::Clip {
                    node_index: AnimationNodeIndex::new(0), // Will be updated by build_animation_nodes
                },
                None,
            )
        }
        EnumClipData::ConditionFloatClipData(ConditionFloatClipData {
            m_condition_float_pair_data_list,
            updater,
            ..
        }) => (
            ConfigAnimationNode::ConditionFloat {
                conditions: m_condition_float_pair_data_list
                    .iter()
                    .map(|v| ConfigAnimationNodeF32 {
                        key: v.m_clip_name,
                        value: v.m_value.unwrap_or(0.0),
                    })
                    .collect(),
                updater: parametric_updater_to_config(updater),
            },
            None,
        ),
        EnumClipData::SelectorClipData(SelectorClipData {
            m_selector_pair_data_list,
            ..
        }) => (
            ConfigAnimationNode::Selector {
                probably_nodes: m_selector_pair_data_list
                    .iter()
                    .map(|v| ConfigAnimationNodeF32 {
                        key: v.m_clip_name,
                        value: v.m_probability.unwrap_or(0.0),
                    })
                    .collect(),
            },
            None,
        ),
        EnumClipData::SequencerClipData(SequencerClipData {
            m_clip_name_list, ..
        }) => (
            ConfigAnimationNode::Sequence {
                hashes: m_clip_name_list.clone(),
            },
            None,
        ),
        EnumClipData::ConditionBoolClipData(ConditionBoolClipData {
            updater,
            m_true_condition_clip_name,
            m_false_condition_clip_name,
            ..
        }) => (
            ConfigAnimationNode::ConditionBool {
                updater: parametric_updater_to_config(updater),
                true_node: *m_true_condition_clip_name,
                false_node: *m_false_condition_clip_name,
            },
            None,
        ),
        _ => (
            ConfigAnimationNode::Clip {
                node_index: AnimationNodeIndex::new(0),
            },
            None,
        ),
    }
}

/// Convert AnimationGraphData to ConfigAnimation
pub fn animation_graph_to_config(
    graph_data: &AnimationGraphData,
    node_index_map: &HashMap<u32, AnimationNodeIndex>,
) -> ConfigAnimation {
    let mut hash_to_node = HashMap::new();

    // Convert clip data map to nodes
    if let Some(clip_data_map) = &graph_data.m_clip_data_map {
        for (hash, clip) in clip_data_map {
            let (node, _) = clip_data_to_node(*hash, clip);
            // Update Clip nodes with correct node_index
            if let ConfigAnimationNode::Clip { node_index } = &mut node.clone() {
                *node_index = *node_index_map.get(hash).unwrap_or(node_index);
            }
            hash_to_node.insert(*hash, node);
        }
    }

    // Add Attack selector node (same as in lol_render)
    hash_to_node.insert(
        hash_bin("Attack"),
        ConfigAnimationNode::Selector {
            probably_nodes: vec![
                ConfigAnimationNodeF32 {
                    key: hash_bin("Attack1"),
                    value: 1.0,
                },
                ConfigAnimationNodeF32 {
                    key: hash_bin("Attack2"),
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
            blend_data.insert((high, low), blend_data_to_config(value));
        }
    }

    ConfigAnimation {
        hash_to_node,
        blend_data,
    }
}
