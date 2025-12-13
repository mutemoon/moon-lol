use std::collections::HashMap;

use bevy::prelude::*;use lol_config::LoadHashKeyTrait;
use league_core::{
    AnimationGraphData, AtomicClipData, ConditionBoolClipData, ConditionFloatClipData,
    EnumClipData, SelectorClipData, SequencerClipData, SkinCharacterDataProperties,
};
use league_to_lol::load_animation_map;
use league_utils::hash_bin;
use lol_config::HashKey;

use crate::{Animation, AnimationNode, AnimationNodeF32, AnimationState, Loading, Skin};

#[derive(EntityEvent)]
pub struct CommandSkinAnimationSpawn {
    pub entity: Entity,
}

#[derive(TypePath)]
pub struct SkinAnimationSpawn(pub HashKey<AnimationGraphData>);

pub fn on_command_skin_animation_spawn(
    trigger: On<CommandSkinAnimationSpawn>,
    mut commands: Commands,
    res_assets_skin_character_data_properties: Res<Assets<SkinCharacterDataProperties>>,
        q_skin: Query<&Skin>,
) {
    let entity = trigger.event_target();

    let skin = q_skin.get(entity).unwrap();

    let skin_character_data_properties = res_assets_skin_character_data_properties.load_hash( skin.key)
        .unwrap();

    commands
        .entity(entity)
        .insert(Loading::new(SkinAnimationSpawn(
            skin_character_data_properties
                .skin_animation_properties
                .animation_graph_data
                .into(),
        )));
}

pub fn update_skin_animation_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut res_animation_graph: ResMut<Assets<AnimationGraph>>,
    res_assets_animation_graph_data: Res<Assets<AnimationGraphData>>,
        q_loading_animation: Query<(Entity, &Loading<SkinAnimationSpawn>)>,
) {
    for (entity, loading) in q_loading_animation.iter() {
        let Some(animation_graph_data) =
            res_assets_animation_graph_data.load_hash( loading.0)
        else {
            continue;
        };

        let (animation_map, blend_data) = load_animation_map(animation_graph_data.clone()).unwrap();

        let mut animation_graph = AnimationGraph::new();

        let hash_to_node =
            build_animation_nodes(animation_map, &asset_server, &mut animation_graph);

        let graph_handle = res_animation_graph.add(animation_graph);

        commands
            .entity(entity)
            .insert((
                AnimationPlayer::default(),
                AnimationGraphHandle(graph_handle),
                Animation {
                    hash_to_node,
                    blend_data,
                },
                AnimationState {
                    last_hash: None,
                    current_hash: hash_bin("Idle1"),
                    current_duration: None,
                    repeat: true,
                },
            ))
            .remove::<Loading<SkinAnimationSpawn>>();
    }
}

fn build_animation_nodes(
    animation_map: HashMap<u32, EnumClipData>,
    asset_server: &Res<AssetServer>,
    animation_graph: &mut AnimationGraph,
) -> HashMap<u32, AnimationNode> {
    let mut hash_to_node = HashMap::new();

    for (hash, clip) in &animation_map {
        match clip {
            EnumClipData::AtomicClipData(AtomicClipData {
                m_animation_resource_data,
                ..
            }) => {
                let clip =
                    asset_server.load(m_animation_resource_data.m_animation_file_path.clone());
                let node_index = animation_graph.add_clip(clip, 1.0, animation_graph.root);
                hash_to_node.insert(*hash, AnimationNode::Clip { node_index });
            }
            EnumClipData::ConditionFloatClipData(ConditionFloatClipData {
                m_condition_float_pair_data_list,
                updater,
                ..
            }) => {
                hash_to_node.insert(
                    *hash,
                    AnimationNode::ConditionFloat {
                        conditions: m_condition_float_pair_data_list
                            .iter()
                            .map(|v| (v.m_clip_name, v.m_value.unwrap_or(0.0)))
                            .map(|(key, value)| AnimationNodeF32 { key, value })
                            .collect::<Vec<_>>(),
                        updater: updater.clone(),
                    },
                );
            }
            EnumClipData::SelectorClipData(SelectorClipData {
                m_selector_pair_data_list,
                ..
            }) => {
                hash_to_node.insert(
                    *hash,
                    AnimationNode::Selector {
                        probably_nodes: m_selector_pair_data_list
                            .iter()
                            .map(|v| (v.m_clip_name, v.m_probability.unwrap_or(0.0)))
                            .map(|(key, value)| AnimationNodeF32 { key, value })
                            .collect::<Vec<_>>(),
                        current_index: None,
                    },
                );
            }
            EnumClipData::SequencerClipData(SequencerClipData {
                m_clip_name_list, ..
            }) => {
                hash_to_node.insert(
                    *hash,
                    AnimationNode::Sequence {
                        hashes: m_clip_name_list.clone(),
                        current_index: None,
                    },
                );
            }
            EnumClipData::ConditionBoolClipData(ConditionBoolClipData {
                updater,
                m_true_condition_clip_name,
                m_false_condition_clip_name,
                ..
            }) => {
                hash_to_node.insert(
                    *hash,
                    AnimationNode::ConditionBool {
                        updater: updater.clone(),
                        true_node: *m_true_condition_clip_name,
                        false_node: *m_false_condition_clip_name,
                    },
                );
            }
            _ => {}
        };
    }

    hash_to_node.insert(
        hash_bin("Attack"),
        AnimationNode::Selector {
            probably_nodes: vec![
                AnimationNodeF32 {
                    key: hash_bin("Attack1"),
                    value: 1.0,
                },
                AnimationNodeF32 {
                    key: hash_bin("Attack2"),
                    value: 1.0,
                },
            ],
            current_index: None,
        },
    );

    hash_to_node
}
