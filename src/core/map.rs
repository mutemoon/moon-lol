use std::collections::HashMap;
use std::f32;

use crate::core::{
    Animation, AnimationNode, AnimationNodeF32, AnimationState, CommandBehaviorAttack,
    CommandBehaviorMoveTo, ConfigCharacterSkinAnimation, ConfigMap, Controller, Team,
};
use crate::core::{ConfigCharacterSkin, ConfigGeometryObject};
use crate::league::LeagueLoader;
use bevy::animation::{AnimationTarget, AnimationTargetId};
use bevy::asset::uuid::Uuid;
use bevy::prelude::*;
use bevy::render::mesh::skinning::SkinnedMesh;

// 基于相机配置的地图边界
pub const MAP_WIDTH: f32 = 14400.0; // cam_MaxX
pub const MAP_HEIGHT: f32 = 14765.0; // cam_MaxY

// 基于相机配置的地图偏移
pub const MAP_OFFSET_X: f32 = 300.0; // cam_MinX
pub const MAP_OFFSET_Y: f32 = 520.0; // cam_MinY

#[derive(Component)]
pub struct Map;

#[derive(Default)]
pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin);
        app.add_systems(Startup, setup);
        app.add_systems(Update, on_map_move);
    }
}

fn setup(
    mut commands: Commands,
    mut res_animation_graph: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    configs: Res<ConfigMap>,
) {
    let geo_entity = spawn_geometry_objects_from_configs(&mut commands, &asset_server, &configs);

    commands
        .entity(geo_entity)
        .insert((Visibility::Visible, Map))
        .observe(on_click_map);

    let environment_entities = spawn_environment_objects_from_configs(
        &mut commands,
        &mut res_animation_graph,
        &asset_server,
        &configs,
    );

    for entity in environment_entities {
        commands
            .entity(entity)
            .insert((Visibility::Visible, Map, Pickable::IGNORE));
    }
}

/// 从Config中的ConfigEnvironmentObject生成环境对象实体
pub fn spawn_skin_entity(
    commands: &mut Commands,
    res_animation_graph: &mut ResMut<Assets<AnimationGraph>>,
    asset_server: &Res<AssetServer>,
    transform: Transform,
    skin: &ConfigCharacterSkin,
) -> Entity {
    // 加载纹理
    let material_handle: Handle<StandardMaterial> = asset_server.load(skin.material_path.clone());

    // 创建父实体
    let parent_entity = commands
        .spawn(transform.with_scale(transform.scale * skin.skin_scale.unwrap_or(1.0)))
        .id();

    // 构建骨骼实体映射
    let mut index_to_entity = vec![Entity::PLACEHOLDER; skin.joints.len()];

    // 创建骨骼实体
    for (i, joint) in skin.joints.iter().enumerate() {
        let ent = commands
            .spawn((
                joint.transform,
                AnimationTarget {
                    id: AnimationTargetId(Uuid::from_u128(joint.hash as u128)),
                    player: parent_entity,
                },
            ))
            .id();
        index_to_entity[i] = ent;
    }

    // 建立骨骼父子关系
    for (i, joint) in skin.joints.iter().enumerate() {
        if joint.parent_index >= 0 {
            let parent_entity_joint = index_to_entity[joint.parent_index as usize];
            commands
                .entity(parent_entity_joint)
                .add_child(index_to_entity[i]);
        } else {
            commands.entity(parent_entity).add_child(index_to_entity[i]);
        }
    }

    let mut animation_graph = AnimationGraph::new();
    let mut hash_to_node = HashMap::new();

    for (hash, clip) in &skin.animation_map {
        match clip {
            ConfigCharacterSkinAnimation::AtomicClipData { clip_path } => {
                let clip = asset_server.load(clip_path.clone());
                let node_index = animation_graph.add_clip(clip, 1.0, animation_graph.root);
                hash_to_node.insert(*hash, AnimationNode::Clip { node_index });
            }
            ConfigCharacterSkinAnimation::ConditionFloatClipData {
                conditions,
                component_name,
                field_name,
            } => {
                hash_to_node.insert(
                    *hash,
                    AnimationNode::ConditionFloat {
                        component_name: component_name.clone(),
                        field_name: field_name.clone(),
                        conditions: conditions
                            .iter()
                            .map(|(key, value)| AnimationNodeF32 {
                                key: *key,
                                value: *value,
                            })
                            .collect::<Vec<_>>(),
                    },
                );
            }
            ConfigCharacterSkinAnimation::SelectorClipData { probably_nodes } => {
                hash_to_node.insert(
                    *hash,
                    AnimationNode::Selector {
                        probably_nodes: probably_nodes
                            .iter()
                            .map(|(key, value)| AnimationNodeF32 {
                                key: *key,
                                value: *value,
                            })
                            .collect::<Vec<_>>(),
                        current_index: None,
                    },
                );
            }
        };
    }

    let graph_handle = res_animation_graph.add(animation_graph);

    commands.entity(parent_entity).insert((
        AnimationPlayer::default(),
        AnimationGraphHandle(graph_handle),
        Animation { hash_to_node },
        AnimationState {
            last_hash: LeagueLoader::hash_bin("Idle1"),
            current_hash: LeagueLoader::hash_bin("Idle1"),
            current_duration: None,
        },
    ));

    // 加载和创建mesh实体
    for submesh_path in &skin.submesh_paths {
        let mesh_handle = asset_server.load(submesh_path.clone());

        let child = commands
            .spawn((
                Transform::default(),
                Mesh3d(mesh_handle),
                MeshMaterial3d(material_handle.clone()),
                SkinnedMesh {
                    inverse_bindposes: asset_server.load(&skin.inverse_bind_pose_path),
                    joints: skin
                        .joint_influences_indices
                        .iter()
                        .map(|&v| index_to_entity[v as usize])
                        .collect::<Vec<_>>(),
                },
            ))
            .id();
        commands.entity(parent_entity).add_child(child);
    }

    parent_entity
}

pub fn spawn_environment_objects_from_configs(
    commands: &mut Commands,
    res_animation_graph: &mut ResMut<Assets<AnimationGraph>>,
    asset_server: &Res<AssetServer>,
    configs: &ConfigMap,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    for (_, environment_object) in &configs.environment_objects {
        let entity = spawn_skin_entity(
            commands,
            res_animation_graph,
            asset_server,
            Transform::from_matrix(environment_object.transform),
            configs
                .skins
                .get(&environment_object.definition.skin)
                .unwrap(),
        );
        entities.push(entity);
    }

    entities
}

/// 从Config中的ConfigGeometryObject生成几何对象实体
pub fn spawn_geometry_object(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    transform: Transform,
    config_geo_object: &ConfigGeometryObject,
) -> Entity {
    // 加载纹理
    let material_handle: Handle<StandardMaterial> =
        asset_server.load(config_geo_object.material_path.clone());

    // 加载网格
    let mesh_handle = asset_server.load(config_geo_object.mesh_path.clone());

    // 创建几何对象实体
    commands
        .spawn((
            transform,
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
        ))
        .id()
}

/// 从Configs批量生成所有几何对象
pub fn spawn_geometry_objects_from_configs(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    configs: &ConfigMap,
) -> Entity {
    let geo_entity = commands.spawn(Transform::default()).id();

    for config_geo_object in &configs.geometry_objects {
        let entity = spawn_geometry_object(
            commands,
            asset_server,
            Transform::default(),
            config_geo_object,
        );

        commands.entity(geo_entity).add_child(entity);
    }

    geo_entity
}

pub fn on_click_map(
    click: Trigger<Pointer<Pressed>>,
    mut commands: Commands,
    q_move: Query<Entity, With<Controller>>,
) {
    let Some(position) = click.hit.position else {
        return;
    };
    let targets = q_move.iter().collect::<Vec<Entity>>();

    commands.trigger_targets(CommandBehaviorMoveTo(position.xz()), targets);
}

pub fn on_map_move(
    mut commands: Commands,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut ray_cast: MeshRayCast,
    q_children: Query<&ChildOf>,
    q_controller: Query<(Entity, &Team), With<Controller>>,
    q_map: Query<Entity, With<Map>>,
    q_target: Query<(Entity, &Transform, &Team)>,
    res_input: Res<ButtonInput<KeyCode>>,
    window: Single<&Window>,
) {
    if !res_input.just_pressed(KeyCode::KeyA) {
        return;
    }

    let Some(viewport_position) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = *camera;

    let Ok(ray) = camera.viewport_to_world(camera_transform, viewport_position) else {
        return;
    };

    let filter = |v| {
        for ancestor in q_children.iter_ancestors(v) {
            if q_map.contains(ancestor) {
                return true;
            }
        }
        false
    };

    let mesh_ray_cast_settings = MeshRayCastSettings::default().with_filter(&filter);

    let hits = ray_cast.cast_ray(ray, &mesh_ray_cast_settings);

    let Some(hit) = hits.first() else {
        return;
    };

    let position = hit.1.point;

    for (entity, team) in q_controller.iter() {
        let mut min_distance = f32::MAX;
        let mut target = None;

        for (entity, transform, target_team) in q_target.iter() {
            let distance = position.distance(transform.translation);
            if distance < min_distance && target_team != team {
                min_distance = distance;
                target = Some(entity);
            }
        }

        let Some(target) = target else {
            continue;
        };

        commands
            .entity(entity)
            .trigger(CommandBehaviorAttack { target });
    }
}
