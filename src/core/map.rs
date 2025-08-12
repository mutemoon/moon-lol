use crate::core::Configs;
use crate::core::{ConfigEnvironmentObject, ConfigGeometryObject};
use crate::core::{MovementDestination, Target};
use bevy::animation::{AnimationTarget, AnimationTargetId};
use bevy::asset::uuid::Uuid;
use bevy::color::palettes;
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

fn setup(mut commands: Commands, configs: Res<Configs>, asset_server: Res<AssetServer>) {
    spawn_geometry_objects_from_configs(&mut commands, &asset_server, &configs);

    spawn_environment_objects_from_configs(&mut commands, &asset_server, &configs);
}

pub fn draw_attack(
    mut gizmos: Gizmos,
    q_attack: Query<(&Transform, &Target)>,
    q_movement_destination: Query<(&Transform, &MovementDestination)>,
    q_target: Query<(&Transform, &Target)>,
    q_transform: Query<&Transform>,
) {
    for (transform, target) in q_attack.iter() {
        let Ok(target_transform) = q_transform.get(target.0) else {
            continue;
        };
        gizmos.line(
            transform.translation,
            target_transform.translation,
            Color::Srgba(palettes::tailwind::RED_500),
        );
    }

    for (transform, movement_destination) in q_movement_destination.iter() {
        let destination = movement_destination.0;

        gizmos.line(
            transform.translation,
            transform
                .translation
                .clone()
                .with_x(destination.x)
                .with_z(destination.y),
            Color::Srgba(palettes::tailwind::GREEN_500),
        );
    }

    for (transform, target) in q_target.iter() {
        let Ok(target_transform) = q_transform.get(target.0) else {
            continue;
        };
        gizmos.line(
            transform.translation,
            target_transform.translation,
            Color::Srgba(palettes::tailwind::YELLOW_500),
        );
    }
}

/// 从Config中的ConfigEnvironmentObject生成环境对象实体
pub fn spawn_environment_object(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    transform: Transform,
    config_env_object: &ConfigEnvironmentObject,
) -> Entity {
    // 加载纹理
    let material_handle: Handle<StandardMaterial> =
        asset_server.load(config_env_object.material_path.clone());

    // 创建父实体
    let parent_entity = commands.spawn(transform).id();

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

    let animation_player = AnimationPlayer::default();

    let animation_graph_handle: Handle<AnimationGraph> =
        asset_server.load(config_env_object.animation_graph_path.clone());

    commands.entity(parent_entity).insert((
        animation_player,
        AnimationGraphHandle(animation_graph_handle),
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
    asset_server: &Res<AssetServer>,
    configs: &Configs,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    for (transform, config_env_object, _) in &configs.environment_objects {
        let entity =
            spawn_environment_object(commands, asset_server, *transform, config_env_object);
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
