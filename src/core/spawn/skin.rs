use std::collections::HashMap;

use bevy::animation::{AnimationTarget, AnimationTargetId};
use bevy::asset::uuid::Uuid;
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::prelude::*;

use bevy::render::render_resource::Face;
use league_core::{
    AnimationGraphData, AnimationGraphDataMClipDataMap, AtomicClipData, ConditionBoolClipData,
    ConditionFloatClipData, SelectorClipData, SequencerClipData, SkinCharacterDataProperties,
};
use league_file::{LeagueSkeleton, LeagueSkinnedMesh};
use league_to_lol::{load_animation_map, skinned_mesh_to_intermediate};
use league_utils::{get_asset_id_by_hash, get_asset_id_by_path, hash_bin, hash_joint};

use crate::{Animation, AnimationNode, AnimationNodeF32, AnimationState};

// 皮肤系统插件
#[derive(Default)]
pub struct PluginSkin;

// 皮肤缩放组件
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct SkinScale(pub f32);

impl Plugin for PluginSkin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_skin_spawn);
        app.add_observer(on_command_spawn_mesh);
        app.add_observer(on_command_spawn_animation);
        app.add_systems(Update, update_skin_scale);
    }
}

#[derive(EntityEvent)]
pub struct CommandSkinSpawn {
    pub entity: Entity,
    pub skin_key: AssetId<SkinCharacterDataProperties>,
}

#[derive(EntityEvent)]
pub struct CommandSpawnMesh {
    pub entity: Entity,
    pub skin_key: AssetId<SkinCharacterDataProperties>,
}

#[derive(EntityEvent)]
pub struct CommandSpawnAnimation {
    pub entity: Entity,
    pub skin_key: AssetId<SkinCharacterDataProperties>,
}

/// 更新皮肤缩放系统
fn update_skin_scale(mut query: Query<(&SkinScale, &mut Transform)>) {
    for (skin_scale, mut transform) in query.iter_mut() {
        transform.scale = Vec3::splat(skin_scale.0);
    }
}

// 皮肤生成命令处理器
fn on_command_skin_spawn(
    trigger: On<CommandSkinSpawn>,
    mut commands: Commands,
    res_assets_skin_character_data_properties: Res<Assets<SkinCharacterDataProperties>>,
) {
    let entity = trigger.event_target();

    // 从 skin_path 获取 ConfigCharacterSkin
    let skin = res_assets_skin_character_data_properties
        .get(trigger.skin_key)
        .unwrap_or_else(|| panic!("Skin not found: {}", trigger.skin_key));

    commands.entity(entity).insert((
        Visibility::default(),
        SkinScale(
            skin.skin_mesh_properties
                .as_ref()
                .unwrap()
                .skin_scale
                .unwrap_or(1.0),
        ),
    ));

    commands.trigger(CommandSpawnMesh {
        entity,
        skin_key: trigger.skin_key.clone(),
    });
}

struct ConfigJoint {
    hash: u32,
    transform: Transform,
    parent_index: i16,
}

fn on_command_spawn_mesh(
    trigger: On<CommandSpawnMesh>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut res_assets_mesh: ResMut<Assets<Mesh>>,
    res_assets_skin_character_data_properties: Res<Assets<SkinCharacterDataProperties>>,
    mut res_assets_standard_material: ResMut<Assets<StandardMaterial>>,
    mut res_assets_league_skeleton: ResMut<Assets<LeagueSkeleton>>,
    mut res_assets_league_skinned_mesh: ResMut<Assets<LeagueSkinnedMesh>>,
    mut res_assets_skinned_mesh_inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    let entity = trigger.event_target();
    let skin_path = &trigger.skin_key;
    let skin_character_data_properties = res_assets_skin_character_data_properties
        .get(trigger.skin_key)
        .unwrap();

    let skin_mesh_properties = skin_character_data_properties
        .skin_mesh_properties
        .as_ref()
        .unwrap();

    let texture = skin_mesh_properties.texture.clone().unwrap();

    let material_handle = get_standard(
        &mut res_assets_standard_material,
        &asset_server,
        Some(texture),
    );

    let league_skeleton = res_assets_league_skeleton
        .get(get_asset_id_by_path(
            skin_mesh_properties.skeleton.as_ref().unwrap(),
        ))
        .unwrap();

    let inverse_bindposes = res_assets_skinned_mesh_inverse_bindposes.add(
        league_skeleton
            .modern_data
            .influences
            .iter()
            .map(|&v| league_skeleton.modern_data.joints[v as usize].inverse_bind_transform)
            .collect::<Vec<_>>(),
    );

    let joints = league_skeleton
        .modern_data
        .joints
        .iter()
        .map(|joint| ConfigJoint {
            hash: hash_joint(&joint.name),
            transform: Transform::from_matrix(joint.local_transform),
            parent_index: joint.parent_index,
        })
        .collect::<Vec<_>>();

    let joint_influences_indices = &league_skeleton.modern_data.influences;

    let mut index_to_entity = vec![Entity::PLACEHOLDER; joints.len()];

    for (i, joint) in joints.iter().enumerate() {
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

    for (i, joint) in joints.iter().enumerate() {
        if joint.parent_index >= 0 {
            let parent_entity_joint = index_to_entity[joint.parent_index as usize];
            commands
                .entity(parent_entity_joint)
                .add_child(index_to_entity[i]);
        } else {
            commands.entity(entity).add_child(index_to_entity[i]);
        }
    }

    let joints = joint_influences_indices
        .iter()
        .map(|&v| index_to_entity[v as usize])
        .collect::<Vec<_>>();

    let skinned_mesh = SkinnedMesh {
        inverse_bindposes,
        joints,
    };

    let league_skinned_mesh = res_assets_league_skinned_mesh
        .get(get_asset_id_by_path(
            skin_mesh_properties.simple_skin.as_ref().unwrap(),
        ))
        .unwrap();

    for (i, _) in league_skinned_mesh.ranges.iter().enumerate() {
        let mesh = skinned_mesh_to_intermediate(&league_skinned_mesh, i);
        let mesh_handle = res_assets_mesh.add(mesh);
        commands.entity(entity).with_child((
            Transform::default(),
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle.clone()),
            skinned_mesh.clone(),
        ));
    }

    commands.entity(entity).insert(skinned_mesh.clone());

    commands.trigger(CommandSpawnAnimation {
        entity,
        skin_key: skin_path.clone(),
    });
}

pub fn get_standard(
    res_assets_standard_material: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    texture: Option<String>,
) -> Handle<StandardMaterial> {
    let material_handle = res_assets_standard_material.add(StandardMaterial {
        base_color_texture: texture.map(|v| asset_server.load(v)),
        unlit: true,
        cull_mode: Some(Face::Front),
        alpha_mode: AlphaMode::Mask(0.3),
        ..default()
    });

    material_handle
}

fn on_command_spawn_animation(
    trigger: On<CommandSpawnAnimation>,
    mut commands: Commands,
    mut res_animation_graph: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    res_assets_skin_character_data_properties: Res<Assets<SkinCharacterDataProperties>>,
    res_assets_animation_graph_data: Res<Assets<AnimationGraphData>>,
) {
    let entity = trigger.event_target();
    let skin_character_data_properties = res_assets_skin_character_data_properties
        .get(trigger.skin_key)
        .unwrap();

    let animation_graph_data = res_assets_animation_graph_data
        .get(get_asset_id_by_hash(
            skin_character_data_properties
                .skin_animation_properties
                .animation_graph_data,
        ))
        .unwrap();

    let (animation_map, blend_data) = load_animation_map(animation_graph_data.clone()).unwrap();

    let mut animation_graph = AnimationGraph::new();

    let hash_to_node = build_animation_nodes(animation_map, &asset_server, &mut animation_graph);

    let graph_handle = res_animation_graph.add(animation_graph);

    commands.entity(entity).insert((
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
    ));
}

fn build_animation_nodes(
    animation_map: HashMap<u32, AnimationGraphDataMClipDataMap>,
    asset_server: &Res<AssetServer>,
    animation_graph: &mut AnimationGraph,
) -> HashMap<u32, AnimationNode> {
    let mut hash_to_node = HashMap::new();

    for (hash, clip) in &animation_map {
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
                            .map(|(key, value)| AnimationNodeF32 { key, value })
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
                            .map(|(key, value)| AnimationNodeF32 { key, value })
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

    hash_to_node
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
