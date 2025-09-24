use bevy::{prelude::*, render::mesh::VertexAttributeValues};

use crate::core::particle::{ATTRIBUTE_LIFETIME, ATTRIBUTE_WORLD_POSITION};

#[derive(Component)]
pub struct ParticleLifeState {
    pub timer: Timer,
}

pub fn update_particle(
    mut commands: Commands,
    mut res_material: ResMut<Assets<Mesh>>,
    mut query: Query<(
        Entity,
        &ChildOf,
        &Transform,
        &Mesh3d,
        &mut ParticleLifeState,
    )>,
    q_global_transform: Query<&GlobalTransform>,
    time: Res<Time>,
) {
    for (entity, parent, transform, mesh_material, mut particle) in query.iter_mut() {
        let mesh = res_material.get_mut(mesh_material).unwrap();
        particle.timer.tick(time.delta());

        if particle.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let lifetime = particle.timer.elapsed_secs() / particle.timer.duration().as_secs_f32();

        let lifetime_values = mesh.attribute_mut(ATTRIBUTE_LIFETIME).unwrap();

        match lifetime_values {
            VertexAttributeValues::Float32x2(items) => {
                for item in items {
                    item[0] = lifetime;
                }
            }
            _ => panic!(),
        }

        let VertexAttributeValues::Float32x3(postion_values) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap()
        else {
            panic!();
        };

        let parent_translation = q_global_transform.get(parent.0).unwrap().translation();

        let transition = transform.translation + parent_translation;

        let postion_values = postion_values
            .iter_mut()
            .map(|v| {
                [
                    v[0] + transition.x,
                    v[1] + transition.y,
                    v[2] + transition.z,
                ]
            })
            .collect::<Vec<_>>();

        mesh.insert_attribute(ATTRIBUTE_WORLD_POSITION, postion_values);
    }
}
