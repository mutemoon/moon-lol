use bevy::math::bounding::{Aabb3d, IntersectsVolume};
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use lol_core::map::MapGeometry;

use crate::environment::unlit_decal::ParticleMaterialUnlitDecal;

#[derive(Component, Default)]
pub struct ParticleDecal {
    visible: HashSet<Entity>,
}

#[derive(Component)]
pub struct ParticleDecalGeometry(pub Entity);

pub fn update_decal_intersections(
    mut commands: Commands,
    mut q_decals: Query<(
        Entity,
        &MeshMaterial3d<ParticleMaterialUnlitDecal>,
        &mut ParticleDecal,
    )>,
    q_map_geo: Query<(Entity, &Mesh3d, &MapGeometry)>,
    q_particle_decal_geometry: Query<(Entity, &ParticleDecalGeometry)>,
    q_global_transform: Query<&GlobalTransform>,
) {
    for (particle_decal_entity, material, mut particle_decal) in q_decals.iter_mut() {
        let Ok(particle_decal_global_transform) = q_global_transform.get(particle_decal_entity)
        else {
            continue;
        };

        let current_bounding_box = Aabb3d::new(
            particle_decal_global_transform.translation(),
            particle_decal_global_transform.scale(),
        );

        for (geometry_entity, mesh3d, map_geometry) in q_map_geo.iter() {
            if current_bounding_box.intersects(&map_geometry.bounding_box) {
                if !particle_decal.visible.contains(&geometry_entity) {
                    commands.spawn((
                        mesh3d.clone(),
                        material.clone(),
                        Pickable::IGNORE,
                        ParticleDecalGeometry(particle_decal_entity),
                    ));
                    particle_decal.visible.insert(geometry_entity);
                }
            } else {
                particle_decal.visible.remove(&geometry_entity);
            }
        }
    }

    for (decal_entity, decal_geometry) in q_particle_decal_geometry.iter() {
        if q_decals.get(decal_geometry.0).is_err() {
            commands.entity(decal_entity).despawn();
        }
    }
}
