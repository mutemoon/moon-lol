use std::collections::HashMap;

use crate::core::{Animation, Configs};
use crate::core::{ConfigEnvironmentObject, ConfigGeometryObject};
use bevy::animation::{AnimationTarget, AnimationTargetId};
use bevy::asset::uuid::Uuid;
use bevy::prelude::*;
use bevy::render::mesh::skinning::SkinnedMesh;

pub const MAP_WIDTH: f32 = 17000.0;
pub const MAP_HEIGHT: f32 = 17000.0;

pub const MAP_OFFSET_X: f32 = 500.0;
pub const MAP_OFFSET_Y: f32 = 500.0;

#[derive(Component)]
#[require(Visibility)]
pub struct Map;

pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    mut res_animation_graph: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    configs: Res<Configs>,
) {
    spawn_geometry_objects_from_configs(&mut commands, &asset_server, &configs);

    spawn_environment_objects_from_configs(
        &mut commands,
        &mut res_animation_graph,
        &asset_server,
        &configs,
    );
}

/// 从Config中的ConfigEnvironmentObject生成环境对象实体
pub fn spawn_environment_object(
    commands: &mut Commands,
    res_animation_graph: &mut ResMut<Assets<AnimationGraph>>,
    asset_server: &Res<AssetServer>,
    transform: Transform,
    config_env_object: &ConfigEnvironmentObject,
) -> Entity {
    // 加载纹理
    let material_handle: Handle<StandardMaterial> =
        asset_server.load(config_env_object.material_path.clone());

    // 创建父实体
    let parent_entity = commands
        .spawn(transform.with_scale(transform.scale * config_env_object.skin_scale.unwrap_or(1.0)))
        .id();

    // 构建骨骼实体映射
    let mut index_to_entity = vec![Entity::PLACEHOLDER; config_env_object.joints.len()];

    // 创建骨骼实体
    for (i, joint) in config_env_object.joints.iter().enumerate() {
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
    for (i, joint) in config_env_object.joints.iter().enumerate() {
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
    let mut hash_to_node_index = HashMap::new();

    for (hash, clip_path) in &config_env_object.clip_map {
        let clip = asset_server.load(clip_path.clone());
        let node_index = animation_graph.add_clip(clip, 1.0, animation_graph.root);
        hash_to_node_index.insert(*hash, node_index);
    }

    let graph_handle = res_animation_graph.add(animation_graph);

    commands.entity(parent_entity).insert((
        AnimationPlayer::default(),
        Animation { hash_to_node_index },
        AnimationGraphHandle(graph_handle),
    ));

    // 加载和创建mesh实体
    for submesh_path in &config_env_object.submesh_paths {
        let mesh_handle = asset_server.load(submesh_path.clone());

        let child = commands
            .spawn((
                Transform::default(),
                Mesh3d(mesh_handle),
                MeshMaterial3d(material_handle.clone()),
                SkinnedMesh {
                    inverse_bindposes: asset_server.load(&config_env_object.inverse_bind_pose_path),
                    joints: config_env_object
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

/// 从Configs批量生成所有环境对象
pub fn spawn_environment_objects_from_configs(
    commands: &mut Commands,
    res_animation_graph: &mut ResMut<Assets<AnimationGraph>>,
    asset_server: &Res<AssetServer>,
    configs: &Configs,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    for (transform, config_env_object, _) in &configs.environment_objects {
        let entity = spawn_environment_object(
            commands,
            res_animation_graph,
            asset_server,
            *transform,
            config_env_object,
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
    configs: &Configs,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    for config_geo_object in &configs.geometry_objects {
        let entity = spawn_geometry_object(
            commands,
            asset_server,
            Transform::from_scale(Vec3::new(1.0, 1.0, -1.0)),
            config_geo_object,
        );
        entities.push(entity);
    }

    entities
}
