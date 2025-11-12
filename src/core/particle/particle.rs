mod mesh;
mod quad;
mod quad_slice;

pub use mesh::*;
pub use quad::*;
pub use quad_slice::*;

use bevy::{
    prelude::*,
    render::mesh::{
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
        VertexAttributeValues,
    },
};

use league_core::VfxEmitterDefinitionDataPrimitive;
use lol_config::ConfigMap;

use crate::{
    Lifetime, ParticleEmitterState, ParticleId, ParticleMaterialSkinnedMeshParticle,
    ParticleMaterialUnlitDecal, ATTRIBUTE_LIFETIME, ATTRIBUTE_WORLD_POSITION,
};

#[derive(Component)]
#[require(Visibility)]
pub struct ParticleState {
    pub birth_uv_offset: Vec2,
    pub birth_uv_scroll_rate: Vec2,
    pub birth_color: Vec4,
    pub birth_scale0: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub frame: f32,
}

pub fn update_particle(
    mut res_mesh: ResMut<Assets<Mesh>>,
    mut res_particle_material_unlit_decal: ResMut<Assets<ParticleMaterialUnlitDecal>>,
    mut res_particle_material_mesh: ResMut<Assets<ParticleMaterialMesh>>,
    res_config_map: Res<ConfigMap>,
    q_particle_state: Query<(
        Entity,
        &Transform,
        &ChildOf,
        &Lifetime,
        &ParticleState,
        &ParticleId,
    )>,
    q_particle_material_unlit_decal: Query<
        &MeshMaterial3d<ParticleMaterialUnlitDecal>,
        With<ParticleState>,
    >,
    q_particle_material_mesh: Query<&MeshMaterial3d<ParticleMaterialMesh>, With<ParticleState>>,
    q_particle_emitter_state: Query<&ParticleEmitterState>,
    q_global_transform: Query<&GlobalTransform>,
    q_mesh3d: Query<&Mesh3d>,
    q_camera_transform: Query<&Transform, With<Camera3d>>,
) {
    for (particle_entity, transform, child_of, lifetime, particle, particle_id) in
        q_particle_state.iter()
    {
        let parent = child_of.parent();

        let life = lifetime.progress();

        let emitter = q_particle_emitter_state.get(parent).unwrap();

        let color = particle.birth_color * emitter.color.sample_clamped(life);

        let emitter_global_transform = q_global_transform.get(parent).unwrap().compute_transform();

        let mut world_transform = emitter_global_transform.mul_transform(*transform);

        let world_matrix = world_transform.compute_matrix();

        let vfx_emitter_definition_data = particle_id.get_def(&res_config_map);

        if let Ok(material) = q_particle_material_unlit_decal.get(particle_entity) {
            if let Some(material) = res_particle_material_unlit_decal.get_mut(material.0.id()) {
                material.uniforms_vertex.decal_world_to_uv_matrix =
                    Mat4::from_translation(Vec3::splat(0.5)) * world_matrix.inverse();
            }
        }

        if let Ok(material) = q_particle_material_mesh.get(particle_entity) {
            if let Some(material) = res_particle_material_mesh.get_mut(material.0.id()) {
                material.uniforms_vertex.m_world = world_matrix;
                let frame = particle.frame;

                let Vec2 {
                    x: col_num,
                    y: row_num,
                } = vfx_emitter_definition_data.tex_div.unwrap_or(Vec2::ONE);

                let scale_x = 1.0 / col_num;
                let scale_y = 1.0 / row_num;

                let current_col = frame % col_num;
                let current_row = (frame / col_num).floor();

                let current_uv_offset: Vec2 = (particle.birth_uv_offset
                    + particle.birth_uv_scroll_rate * lifetime.elapsed_secs())
                    % 1.0;

                let translate_x = current_col * scale_x;
                let translate_y = current_row * scale_y;

                let final_translate_x = current_uv_offset.x * scale_x + translate_x;
                let final_translate_y = current_uv_offset.y * scale_y + translate_y;

                material.uniforms_vertex.v_particle_uvtransform = [
                    vec3(scale_x, 0., 0.),
                    vec3(0., scale_y, 0.),
                    vec3(final_translate_x, final_translate_y, 0.),
                    Vec3::ZERO,
                ];

                material.uniforms_pixel.color_lookup_uv = vec2(life, life);

                material.uniforms_vertex.k_color_factor = color;
            }
        }

        let Ok(mesh3d) = q_mesh3d.get(particle_entity) else {
            continue;
        };

        let Some(mesh) = res_mesh.get_mut(mesh3d) else {
            continue;
        };

        let Some(lifetime_values) = mesh.attribute_mut(ATTRIBUTE_LIFETIME) else {
            continue;
        };

        match lifetime_values {
            VertexAttributeValues::Float32x2(items) => {
                for item in items {
                    item[0] = life;
                    item[1] = 0.0;
                }
            }
            _ => panic!(),
        }

        let VertexAttributeValues::Float32x3(postion_values) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap()
        else {
            panic!();
        };

        if vfx_emitter_definition_data.primitive.is_none()
            || matches!(
                vfx_emitter_definition_data.primitive,
                Some(VfxEmitterDefinitionDataPrimitive::VfxPrimitiveCameraUnitQuad)
            )
        {
            let camera_transform = q_camera_transform.single().unwrap();
            world_transform = world_transform.looking_at(camera_transform.translation, Vec3::ZERO);
        }

        let postion_values = postion_values
            .iter()
            .map(|v| {
                let vertext_position = Vec3::from_array(*v);
                world_transform.transform_point(vertext_position).to_array()
            })
            .collect::<Vec<_>>();

        mesh.insert_attribute(ATTRIBUTE_WORLD_POSITION, postion_values);

        let VertexAttributeValues::Float32x4(values) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR).unwrap()
        else {
            panic!();
        };

        let values = values.iter().map(|_| color.to_array()).collect::<Vec<_>>();

        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, values);
    }
}

pub fn update_particle_transform(
    mut q_particle_state: Query<(&mut Transform, &ChildOf, &Lifetime, &mut ParticleState)>,
    q_particle_emitter_state: Query<&ParticleEmitterState>,
    res_time: Res<Time>,
) {
    let dt = res_time.delta_secs();

    for (mut transform, child_of, lifetime, mut particle) in q_particle_state.iter_mut() {
        particle.velocity = particle.velocity + particle.acceleration * dt;

        transform.translation += particle.velocity * dt;

        let parent = child_of.parent();

        let life = lifetime.progress();

        let emitter = q_particle_emitter_state.get(parent).unwrap();

        let scale0 = emitter.scale0.sample_clamped(life);

        transform.scale = scale0 * particle.birth_scale0;
    }
}

pub fn update_particle_skinned_mesh_particle(
    mut res_particle_material_skinned_mesh_particle: ResMut<
        Assets<ParticleMaterialSkinnedMeshParticle>,
    >,
    res_inverse_bindposes: Res<Assets<SkinnedMeshInverseBindposes>>,
    q_particle_state: Query<(
        Entity,
        &ChildOf,
        &Lifetime,
        &ParticleState,
        &MeshMaterial3d<ParticleMaterialSkinnedMeshParticle>,
    )>,
    q_particle_emitter_state: Query<&ParticleEmitterState>,
    q_global_transform: Query<&GlobalTransform>,
    q_skinned_mesh: Query<&SkinnedMesh>,
) {
    for (particle_entity, child_of, lifetime, particle, material) in q_particle_state.iter() {
        let parent = child_of.parent();

        let life = lifetime.progress();

        let emitter = q_particle_emitter_state.get(parent).unwrap();

        let color = particle.birth_color * emitter.color.sample_clamped(life);

        let material = res_particle_material_skinned_mesh_particle
            .get_mut(material.0.id())
            .unwrap();

        let skinned_mesh = q_skinned_mesh.get(particle_entity).unwrap();

        let inverse_bindposes = res_inverse_bindposes
            .get(skinned_mesh.inverse_bindposes.id())
            .unwrap();

        let mut bones = Vec::new();

        for (i, entity) in skinned_mesh.joints.iter().enumerate() {
            let g = q_global_transform.get(*entity).unwrap();
            bones.push(g.compute_matrix() * inverse_bindposes[i]);
        }

        let current_uv_offset: Vec2 =
            particle.birth_uv_offset + particle.birth_uv_scroll_rate * lifetime.elapsed_secs();

        material.uniforms_vertex.v_particle_uvtransform = [
            Vec3::X,
            Vec3::Y,
            vec3(current_uv_offset.x, current_uv_offset.y, 0.),
            Vec3::ZERO,
        ];

        material.uniforms_pixel.color_lookup_uv = vec2(life, life);

        material.uniforms_vertex.k_color_factor = color;

        material.uniforms_vertex.bones = mat4_vec_to_mat4_array_homogeneous(bones);

        continue;
    }
}

pub fn mat4_vec_to_mat4_array_homogeneous(mats: Vec<Mat4>) -> [[Vec3; 4]; 68] {
    const IDENTITY_VEC3_ARRAY: [Vec3; 4] = [Vec3::X, Vec3::Y, Vec3::Z, Vec3::ZERO];

    let mut bone_array: [[Vec3; 4]; 68] = [IDENTITY_VEC3_ARRAY; 68];

    for (i, mat) in mats.into_iter().enumerate().take(68) {
        let cols = mat.to_cols_array();

        bone_array[i] = [
            Vec3::new(cols[0], cols[1], cols[2]),
            Vec3::new(cols[4], cols[5], cols[6]),
            Vec3::new(cols[8], cols[9], cols[10]),
            Vec3::new(cols[12], cols[13], cols[14]),
        ];
    }

    bone_array
}
