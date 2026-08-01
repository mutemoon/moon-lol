use std::collections::HashMap;

use bevy::animation::AnimationTargetId;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;

pub fn spawn_shadow_skin_entity<M: Material>(
    commands: &mut Commands,
    target: Entity,
    skin_entity: Entity,
    material: MeshMaterial3d<M>,
    q_mesh3d: &Query<&Mesh3d>,
    q_skinned_mesh: &Query<&SkinnedMesh>,
    q_children: &Query<&Children>,
    q_animation_target: &Query<(Entity, &Transform, &AnimationTargetId)>,
    q_parent: &Query<&ChildOf>,
) {
    // 粒子可能挂在英雄根实体或骨骼实体上，蒙皮网格实体不一定是源实体本身；
    // 先向上/向下解析出真正的 SkinnedMesh 实体，找不到则跳过阴影皮肤（不 panic）。
    let Some(skin_entity) = resolve_skinned_entity(skin_entity, q_skinned_mesh, q_children, q_parent)
    else {
        warn!(
            "{skin_entity} 附近未找到 SkinnedMesh，跳过粒子阴影皮肤",
        );
        return;
    };

    let Ok(children) = q_children.get(skin_entity) else {
        warn!("{skin_entity} 蒙皮网格没有 Children，跳过粒子阴影皮肤");
        return;
    };

    let Ok(skinned_mesh) = q_skinned_mesh.get(skin_entity) else {
        return;
    };

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
        q_children,
        q_animation_target,
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
                new_skinned_mesh.clone(),
            ));
        }
    }
}

/// 从源实体出发，先看自身，再沿祖先向上、沿后代向下查找最近的 SkinnedMesh 实体。
fn resolve_skinned_entity(
    entity: Entity,
    q_skinned_mesh: &Query<&SkinnedMesh>,
    q_children: &Query<&Children>,
    q_parent: &Query<&ChildOf>,
) -> Option<Entity> {
    if q_skinned_mesh.get(entity).is_ok() {
        return Some(entity);
    }

    let mut cur = entity;
    while let Ok(parent) = q_parent.get(cur) {
        cur = parent.parent();
        if q_skinned_mesh.get(cur).is_ok() {
            return Some(cur);
        }
    }

    q_children
        .iter_descendants(entity)
        .find(|descendant| q_skinned_mesh.get(*descendant).is_ok())
}

pub fn duplicate_joints_to_target(
    commands: &mut Commands,
    parent: Entity,
    joints: Vec<(Entity, &Transform, &AnimationTargetId)>,
    q_children: &Query<&Children>,
    q_animation_target: &Query<(Entity, &Transform, &AnimationTargetId)>,
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
