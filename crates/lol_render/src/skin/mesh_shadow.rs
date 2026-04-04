use std::collections::HashMap;

use bevy::animation::AnimationTarget;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;

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
