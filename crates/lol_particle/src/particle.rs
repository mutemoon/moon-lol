pub mod assembly;
pub mod dynamic;

use bevy::mesh::VertexAttributeValues;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use lol_base_render::camera::CameraState;
use lol_base_render::particle::{ConfigVfxPrimitive, ConfigVfxSystemDefinition};
use lol_core::lifetime::Lifetime;

use crate::emitters::state::ParticleEmitterState;
use crate::particle::dynamic::{ParticleMaterialDynamic, ParticleRenderKind};
use crate::{
    ATTRIBUTE_LIFETIME, ATTRIBUTE_UV_FRAME, ATTRIBUTE_WORLD_POSITION,
    ATTRIBUTE_WORLD_POSITION_VEC4, ParticleId,
};

#[derive(Component)]
#[require(Visibility)]
pub struct ParticleState {
    pub birth_uv_offset: Vec2,
    pub birth_uv_scroll_rate: Vec2,
    pub birth_color: Vec4,
    pub birth_scale0: Vec3,
    pub initial_rotation: Quat,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub frame: f32,
}

pub fn update_particle(
    mut res_mesh: ResMut<Assets<Mesh>>,
    res_particle_material_dynamic: Res<Assets<ParticleMaterialDynamic>>,
    res_assets_vfx_system_definition_data: Res<Assets<ConfigVfxSystemDefinition>>,
    q_particle_state: Query<(
        Entity,
        &Transform,
        &ChildOf,
        &Lifetime,
        &ParticleState,
        &ParticleId,
    )>,
    q_particle_material_dynamic: Query<
        &MeshMaterial3d<ParticleMaterialDynamic>,
        With<ParticleState>,
    >,
    q_particle_emitter_state: Query<&ParticleEmitterState>,
    q_global_transform: Query<&GlobalTransform>,
    q_mesh3d: Query<&Mesh3d>,
    q_camera_transform: Query<&Transform, With<CameraState>>,
    mut commands: Commands,
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

        let world_matrix = world_transform.to_matrix();

        let vfx_emitter_definition_data =
            particle_id.get_def(&res_assets_vfx_system_definition_data);

        // ── 动态材质逐帧更新：Mesh 粒子的动态 uniform 走 ECS 组件，供 Render World 原地 DMA 更新 ──
        // 彻底免去调用 res_particle_material_dynamic.get_mut 触发 AssetEvent::Modified 导致的 GPU 重建
        if let Ok(material_handle) = q_particle_material_dynamic.get(particle_entity) {
            if let Some(material) = res_particle_material_dynamic.get(material_handle.0.id()) {
                if material.kind == ParticleRenderKind::Mesh {
                    let frame = particle.frame;

                    let Vec2 {
                        x: col_num,
                        y: row_num,
                    } = vfx_emitter_definition_data.tex_div.unwrap_or(Vec2::ONE);

                    let scale = vec2(1.0 / col_num, 1.0 / row_num);

                    let current_col = frame % col_num;
                    let current_row = (frame / col_num).floor();

                    let current_uv_offset: Vec2 = (particle.birth_uv_offset
                        + particle.birth_uv_scroll_rate * lifetime.elapsed_secs())
                        % 1.0;

                    let translate =
                        current_uv_offset * scale + vec2(current_col, current_row) * scale;

                    commands.entity(particle_entity).insert(ParticleDynamicUniforms {
                        world_matrix_transpose: world_matrix.transpose(),
                        uv_transform: uv_transform_rows(scale, translate),
                        color_factor: color,
                        color_lookup_uv: vec2(life, life),
                    });
                }
            }
        }

        let Ok(mesh3d) = q_mesh3d.get(particle_entity) else {
            continue;
        };

        let Some(mut mesh) = res_mesh.get_mut(mesh3d) else {
            continue;
        };

        if let Some(VertexAttributeValues::Float32x4(values)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
        {
            let values = values
                .iter()
                .map(|_| [color.z, color.y, color.x, color.w])
                .collect::<Vec<_>>();

            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, values);
        }

        if let Some(VertexAttributeValues::Float32x2(items)) =
            mesh.attribute_mut(ATTRIBUTE_LIFETIME)
        {
            for item in items {
                item[0] = life;
                item[1] = 0.0;
            }
        }

        if let Some(VertexAttributeValues::Float32x4(uv_frame_values)) =
            mesh.attribute_mut(ATTRIBUTE_UV_FRAME)
        {
            let erosion_drive =
                if let Some(erosion) = &vfx_emitter_definition_data.alpha_erosion_definition {
                    erosion.erosion_drive_curve.sample_clamped(life)
                } else {
                    life
                };

            for item in uv_frame_values {
                item[3] = erosion_drive;
            }
        }

        let is_vec4_world_pos = mesh.contains_attribute(ATTRIBUTE_WORLD_POSITION_VEC4);
        let is_vec3_world_pos = mesh.contains_attribute(ATTRIBUTE_WORLD_POSITION);

        if let Some(VertexAttributeValues::Float32x3(postion_values)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            if vfx_emitter_definition_data.primitive.is_none()
                || matches!(
                    vfx_emitter_definition_data.primitive,
                    Some(ConfigVfxPrimitive::VfxPrimitiveCameraUnitQuad)
                )
            {
                let camera_transform = q_camera_transform.single().unwrap();
                world_transform =
                    world_transform.looking_at(camera_transform.translation, Vec3::ZERO);
            }

            if is_vec4_world_pos {
                // Distortion particles use ATTRIBUTE_WORLD_POSITION_VEC4
                let postion_values = postion_values
                    .iter()
                    .map(|v| {
                        let vertext_position = Vec3::from_array(*v);
                        world_transform
                            .transform_point(vertext_position)
                            .extend(0.0)
                            .to_array()
                    })
                    .collect::<Vec<_>>();

                mesh.insert_attribute(ATTRIBUTE_WORLD_POSITION_VEC4, postion_values);
            } else if is_vec3_world_pos {
                // Other particles use ATTRIBUTE_WORLD_POSITION
                let postion_values = postion_values
                    .iter()
                    .map(|v| {
                        let vertext_position = Vec3::from_array(*v);
                        world_transform.transform_point(vertext_position).to_array()
                    })
                    .collect::<Vec<_>>();

                mesh.insert_attribute(ATTRIBUTE_WORLD_POSITION, postion_values);
            }
        }
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

        if emitter.is_direction_oriented {
            let speed_sq = particle.velocity.length_squared();
            if speed_sq > 0.0001 {
                let dir = particle.velocity / speed_sq.sqrt();
                let q_vel = Quat::from_rotation_arc(Vec3::Z, dir);
                transform.rotation = q_vel * particle.initial_rotation;
            }
        }

        let scale0 = emitter.scale0.sample_clamped(life);

        transform.scale = scale0 * particle.birth_scale0;
    }
}

/// 针对 Mesh 与 SkinnedMesh 粒子的逐帧动态 Uniform 数据。
/// 挂载在粒子实体上，由 Render World 逐帧通过 DMA write_buffer 原地写入 GPU Buffer，
/// 彻底免除主线程调用 get_mut 触发 AssetEvent::Modified 导致的 GPU Buffer / BindGroup 全量重建。
#[derive(Component, Clone, Copy, Default, Debug)]
pub struct ParticleDynamicUniforms {
    pub world_matrix_transpose: Mat4,
    pub uv_transform: [[f32; 4]; 3],
    pub color_factor: Vec4,
    pub color_lookup_uv: Vec2,
}

pub fn update_particle_skinned_mesh_particle(
    mut commands: Commands,
    q_particle_state: Query<
        (
            Entity,
            &ChildOf,
            &Lifetime,
            &ParticleState,
            &MeshMaterial3d<ParticleMaterialDynamic>,
        ),
        With<SkinnedMesh>,
    >,
    q_particle_emitter_state: Query<&ParticleEmitterState>,
) {
    for (particle_entity, child_of, lifetime, particle, _material_handle) in q_particle_state.iter() {
        let parent = child_of.parent();

        let life = lifetime.progress();

        let emitter = q_particle_emitter_state.get(parent).unwrap();

        let color = particle.birth_color * emitter.color.sample_clamped(life);

        let current_uv_offset: Vec2 =
            particle.birth_uv_offset + particle.birth_uv_scroll_rate * lifetime.elapsed_secs();

        commands.entity(particle_entity).insert(ParticleDynamicUniforms {
            world_matrix_transpose: Mat4::IDENTITY,
            uv_transform: uv_transform_rows(Vec2::ONE, current_uv_offset),
            color_factor: color,
            color_lookup_uv: vec2(life, life),
        });
    }
}

/// 把「列描述」的 UV 仿射变换（scale + translate）转成 unified 布局里
/// `vParticleUVTransform` 的 row-major float3x4（3 行 float4，共 48 字节）：
/// rows = (sx, 0, tx, 0) / (0, sy, ty, 0) / (0, 0, 0, 0)。
fn uv_transform_rows(scale: Vec2, translate: Vec2) -> [[f32; 4]; 3] {
    [
        [scale.x, 0.0, translate.x, 0.0],
        [0.0, scale.y, translate.y, 0.0],
        [0.0, 0.0, 0.0, 0.0],
    ]
}
