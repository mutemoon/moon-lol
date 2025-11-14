use std::collections::HashMap;

use bevy::animation::{AnimationTarget, AnimationTargetId};
use bevy::asset::uuid::Uuid;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;

use league_core::{
    AnimationGraphDataMClipDataMap, AtomicClipData, ConditionBoolClipData, ConditionFloatClipData,
    SelectorClipData, SequencerClipData,
};
use league_utils::hash_bin;
use lol_config::ConfigCharacterSkin;

use crate::{Animation, AnimationNode, AnimationNodeF32, AnimationState, ResourceCache};

// 皮肤系统插件
#[derive(Default)]
pub struct PluginSkin;

// 皮肤缩放组件
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct SkinScale(pub f32);

impl Plugin for PluginSkin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_skin_spawn);
        app.add_systems(Update, update_skin_scale);
    }
}

// 皮肤系统事件定义
#[derive(EntityEvent)]
pub struct EventSkinSpawn {
    entity: Entity,
}

#[derive(EntityEvent)]
pub struct EventSkinSpawnComplete {
    entity: Entity,
}

#[derive(EntityEvent)]
pub struct CommandSkinSpawn {
    pub entity: Entity,
    pub skin_path: String,
}

// 皮肤生成命令处理器
fn on_command_skin_spawn(
    trigger: On<CommandSkinSpawn>,
    mut commands: Commands,
    mut res_animation_graph: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    res_resource_cache: Res<ResourceCache>,
) {
    let entity = trigger.event_target();

    commands.trigger(EventSkinSpawn { entity });

    // 从 skin_path 获取 ConfigCharacterSkin
    let skin = res_resource_cache
        .skins
        .get(&trigger.skin_path)
        .unwrap_or_else(|| panic!("Skin not found: {}", trigger.skin_path));

    // 设置初始的 SkinScale 组件
    let skin_scale = SkinScale(skin.skin_scale.unwrap_or(1.0));
    commands.entity(entity).insert(skin_scale);

    spawn_skin_entity(
        &mut commands,
        &mut res_animation_graph,
        &asset_server,
        entity,
        skin,
    );

    commands.trigger(EventSkinSpawnComplete { entity });
}

/// 更新皮肤缩放系统
fn update_skin_scale(mut query: Query<(&SkinScale, &mut Transform), Changed<SkinScale>>) {
    for (skin_scale, mut transform) in query.iter_mut() {
        transform.scale = Vec3::splat(skin_scale.0);
    }
}

fn spawn_skin_entity(
    commands: &mut Commands,
    res_animation_graph: &mut ResMut<Assets<AnimationGraph>>,
    asset_server: &Res<AssetServer>,
    entity: Entity,
    skin: &ConfigCharacterSkin,
) {
    let material_handle: Handle<StandardMaterial> = asset_server.load(skin.material_path.clone());

    commands.entity(entity).insert(Visibility::default());

    let mut index_to_entity = vec![Entity::PLACEHOLDER; skin.joints.len()];

    for (i, joint) in skin.joints.iter().enumerate() {
        let ent = commands
            .spawn((
                joint.transform,
                AnimationTarget {
                    id: AnimationTargetId(Uuid::from_u128(joint.hash as u128)),
                    player: entity,
                },
            ))
            .id();
        index_to_entity[i] = ent;
    }

    for (i, joint) in skin.joints.iter().enumerate() {
        if joint.parent_index >= 0 {
            let parent_entity_joint = index_to_entity[joint.parent_index as usize];
            commands
                .entity(parent_entity_joint)
                .add_child(index_to_entity[i]);
        } else {
            commands.entity(entity).add_child(index_to_entity[i]);
        }
    }

    let mut animation_graph = AnimationGraph::new();
    let mut hash_to_node = HashMap::new();

    for (hash, clip) in &skin.animation_map {
        match clip {
            AnimationGraphDataMClipDataMap::AtomicClipData(AtomicClipData {
                m_animation_resource_data,
                ..
            }) => {
                let clip =
                    asset_server.load(m_animation_resource_data.m_animation_file_path.clone());
                let node_index = animation_graph.add_clip(clip, 1.0, animation_graph.root);
                hash_to_node.insert(*hash, AnimationNode::Clip { node_index });
            }
            AnimationGraphDataMClipDataMap::ConditionFloatClipData(ConditionFloatClipData {
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
                            .map(|(key, value)| AnimationNodeF32 {
                                key: key,
                                value: value,
                            })
                            .collect::<Vec<_>>(),
                        updater: updater.clone(),
                    },
                );
            }
            AnimationGraphDataMClipDataMap::SelectorClipData(SelectorClipData {
                m_selector_pair_data_list,
                ..
            }) => {
                hash_to_node.insert(
                    *hash,
                    AnimationNode::Selector {
                        probably_nodes: m_selector_pair_data_list
                            .iter()
                            .map(|v| (v.m_clip_name, v.m_probability.unwrap_or(0.0)))
                            .map(|(key, value)| AnimationNodeF32 {
                                key: key,
                                value: value,
                            })
                            .collect::<Vec<_>>(),
                        current_index: None,
                    },
                );
            }
            AnimationGraphDataMClipDataMap::SequencerClipData(SequencerClipData {
                m_clip_name_list,
                ..
            }) => {
                hash_to_node.insert(
                    *hash,
                    AnimationNode::Sequence {
                        hashes: m_clip_name_list.clone(),
                        current_index: None,
                    },
                );
            }
            AnimationGraphDataMClipDataMap::ConditionBoolClipData(ConditionBoolClipData {
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

    let graph_handle = res_animation_graph.add(animation_graph);

    commands.entity(entity).insert((
        AnimationPlayer::default(),
        AnimationGraphHandle(graph_handle),
        Animation {
            hash_to_node,
            blend_data: skin.blend_data.clone(),
        },
        AnimationState {
            last_hash: None,
            current_hash: hash_bin("Idle1"),
            current_duration: None,
            repeat: true,
        },
    ));

    let inverse_bindposes = asset_server.load(&skin.inverse_bind_pose_path);
    let joints = skin
        .joint_influences_indices
        .iter()
        .map(|&v| index_to_entity[v as usize])
        .collect::<Vec<_>>();
    let skinned_mesh = SkinnedMesh {
        inverse_bindposes,
        joints,
    };

    for submesh_path in &skin.submesh_paths {
        let mesh_handle = asset_server.load(submesh_path.clone());
        commands.entity(entity).with_child((
            Transform::default(),
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle.clone()),
            skinned_mesh.clone(),
        ));
    }

    commands.entity(entity).insert(skinned_mesh.clone());
}

pub fn spawn_shadow_skin_entity<M: Material>(
    commands: &mut Commands,
    target: Entity,
    skin_entity: Entity,
    material: MeshMaterial3d<M>,
    q_mesh3d: Query<&Mesh3d>,
    q_skinned_mesh: Query<&SkinnedMesh>,
    q_children: Query<&Children>,
    q_animation_target: Query<(Entity, &Transform, &AnimationTarget)>,
) {
    let children = q_children.get(skin_entity).unwrap();

    let skinned_mesh = q_skinned_mesh.get(skin_entity).unwrap();

    commands.entity(target).insert(material.clone());

    let mut joints = Vec::new();

    for child in children.iter() {
        if let Ok(joint) = q_animation_target.get(child) {
            joints.push(joint);
        }
    }

    let mut joint_map: HashMap<Entity, Entity> = HashMap::new();

    duplicate_joints_to_target(
        commands,
        target,
        joints,
        &q_children,
        &q_animation_target,
        &mut joint_map,
    );

    let new_joints = skinned_mesh
        .joints
        .iter()
        .map(|old_joint_entity| *joint_map.get(old_joint_entity).unwrap())
        .collect::<Vec<_>>();

    let new_skinned_mesh = SkinnedMesh {
        inverse_bindposes: skinned_mesh.inverse_bindposes.clone(),
        joints: new_joints,
    };

    commands.entity(target).insert(new_skinned_mesh.clone());

    for child in children.iter() {
        if let Ok(mesh) = q_mesh3d.get(child) {
            commands.entity(target).with_child((
                mesh.clone(),
                material.clone(),
                // skinned_mesh.clone(),
                // mat.clone(),
                new_skinned_mesh.clone(),
            ));
        }
    }
}

pub fn duplicate_joints_to_target(
    commands: &mut Commands,
    parent: Entity,
    joints: Vec<(Entity, &Transform, &AnimationTarget)>,
    q_children: &Query<&Children>,
    q_animation_target: &Query<(Entity, &Transform, &AnimationTarget)>,
    joint_map: &mut HashMap<Entity, Entity>,
) {
    for (joint_entity, transform, anim_target) in joints {
        let new_joint_entity = commands
            .spawn((transform.clone(), anim_target.clone()))
            .id();

        commands.entity(parent).add_child(new_joint_entity);

        joint_map.insert(joint_entity, new_joint_entity);

        if let Ok(children) = q_children.get(joint_entity) {
            let mut joints = Vec::new();

            for child in children {
                if let Ok(joint) = q_animation_target.get(*child) {
                    joints.push(joint);
                }
            }

            duplicate_joints_to_target(
                commands,
                new_joint_entity,
                joints,
                q_children,
                q_animation_target,
                joint_map,
            );
        }
    }
}
