use bevy::animation::{AnimationTarget, AnimationTargetId};
use bevy::asset::uuid::Uuid;
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::prelude::*;
use league_file::LeagueSkeleton;
use league_utils::hash_joint;

use crate::{AssetServerLoadLeague, Loading};

#[derive(EntityEvent)]
pub struct CommandSkinSkeletonSpawn {
    pub entity: Entity,
    pub path: String,
}

#[derive(TypePath)]
pub struct SkinSkeletonSpawn(pub Handle<LeagueSkeleton>);

struct ConfigJoint {
    hash: u32,
    transform: Transform,
    parent_index: i16,
}

pub fn on_command_skin_skeleton_spawn(
    trigger: On<CommandSkinSkeletonSpawn>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let entity = trigger.event_target();

    commands
        .entity(entity)
        .insert(Loading::new(SkinSkeletonSpawn(
            asset_server.load_league(&trigger.path),
        )));
}

pub fn update_skin_skeleton_spawn(
    mut commands: Commands,
    mut res_assets_skinned_mesh_inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    res_assets_league_skeleton: ResMut<Assets<LeagueSkeleton>>,
    q_loading_skeleton: Query<(Entity, &Loading<SkinSkeletonSpawn>)>,
) {
    for (entity, loading) in q_loading_skeleton.iter() {
        let Some(league_skeleton) = res_assets_league_skeleton.get(&loading.0) else {
            continue;
        };

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

        commands
            .entity(entity)
            .insert(skinned_mesh.clone())
            .remove::<Loading<SkinSkeletonSpawn>>();
    }
}
