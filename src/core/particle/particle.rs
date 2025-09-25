use bevy::{prelude::*, render::mesh::VertexAttributeValues};

use crate::core::particle::{ATTRIBUTE_LIFETIME, ATTRIBUTE_WORLD_POSITION};

#[derive(Component)]
pub struct ParticleState {
    pub timer_life: Timer,
    pub is_local_orientation: bool,
}

pub fn update_particle(
    mut commands: Commands,
    mut res_material: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &ChildOf, &Transform, &Mesh3d, &mut ParticleState)>,
    q_global_transform: Query<&GlobalTransform>,
    time: Res<Time>,
) {
    for (entity, parent, transform, mesh_material, mut particle_state) in query.iter_mut() {
        let mesh = res_material.get_mut(mesh_material).unwrap();
        particle_state.timer_life.tick(time.delta());

        if particle_state.timer_life.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let lifetime = particle_state.timer_life.elapsed_secs()
            / particle_state.timer_life.duration().as_secs_f32();

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

        let parent_global_transform = q_global_transform.get(parent.0).unwrap();

        let mut parent_transform = parent_global_transform.compute_transform();

        if !particle_state.is_local_orientation {
            parent_transform.rotation = Quat::default()
        }

        let local_to_world = parent_transform.mul_transform(*transform);

        let postion_values = postion_values
            .iter_mut()
            .map(|v| {
                let vertext_position = Vec3::from_array(*v);
                local_to_world.transform_point(vertext_position).to_array()
            })
            .collect::<Vec<_>>();

        mesh.insert_attribute(ATTRIBUTE_WORLD_POSITION, postion_values);
    }
}
