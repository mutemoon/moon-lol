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
    mut res_particle_material_dynamic: ResMut<Assets<ParticleMaterialDynamic>>,
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
    q_camera: Query<(&Projection, &GlobalTransform), With<CameraState>>,
) {
    // 取带 CameraState 的窗口主相机：PluginCamera 还会生成一个渲染到 512×512 贴图的
    // 子相机（aspect=1），若任取第一个可能拿到它的投影矩阵，导致窗口里粒子 X 向被拉宽
    let cam_data = (|| {
        let (projection, gtransform) = q_camera.single().ok()?;
        let clip_from_view: Mat4 = projection.get_clip_from_view();
        let view_from_world = gtransform.to_matrix().inverse();
        let clip_from_world = clip_from_view * view_from_world;
        let cam_pos = gtransform.translation();
        Some((clip_from_world, cam_pos))
    })();

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

        // ── 动态材质统一逐帧更新：按几何族分支，全部走成员名查表写入 ──
        if let Ok(material_handle) = q_particle_material_dynamic.get(particle_entity) {
            if let Some(mut material) =
                res_particle_material_dynamic.get_mut(material_handle.0.id())
            {
                // 相机参数全族统一写入（族布局无该成员时安全 no-op）
                if let Some((clip_from_world, cam_pos)) = &cam_data {
                    material.set_param("mProj", clip_from_world.transpose());
                    material.set_param("vCamera", *cam_pos);
                }

                // 逐帧更新 Alpha Erosion 驱动值
                material.update_alpha_erosion_params(life);

                match material.kind {
                    ParticleRenderKind::Quad => {}
                    ParticleRenderKind::Mesh => {
                        // mWorld 位于 CharacterPerDrawVertexCB；row-major 存储需转置
                        material.set_param("mWorld", world_matrix.transpose());

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

                        material
                            .set_param("vParticleUVTransform", uv_transform_rows(scale, translate));
                        material.set_param("kColorFactor", color);
                        material.set_param("COLOR_LOOKUP_UV", vec2(life, life));
                    }
                    ParticleRenderKind::Distortion => {
                        // TEXTURE_INFO / 扭曲强度等在发射器装配时一次性写入；
                        // 逐帧只需重写世界坐标顶点属性（见下方按 kind 的 mesh 属性分支）
                    }
                    ParticleRenderKind::SkinnedMesh => {
                        // UV/颜色在 update_particle_skinned_mesh_particle 中处理
                        //（需要 SkinnedMesh 组件与关节全局变换）
                    }
                    ParticleRenderKind::UnlitDecal => {
                        material.set_param(
                            "DECAL_WORLD_TO_UV_MATRIX",
                            (Mat4::from_translation(Vec3::splat(0.5)) * world_matrix.inverse())
                                .transpose(),
                        );
                    }
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

pub fn update_particle_skinned_mesh_particle(
    mut res_particle_material_dynamic: ResMut<Assets<ParticleMaterialDynamic>>,
    q_particle_state: Query<
        (
            &ChildOf,
            &Lifetime,
            &ParticleState,
            &MeshMaterial3d<ParticleMaterialDynamic>,
        ),
        With<SkinnedMesh>,
    >,
    q_particle_emitter_state: Query<&ParticleEmitterState>,
) {
    for (child_of, lifetime, particle, material_handle) in q_particle_state.iter() {
        let parent = child_of.parent();

        let life = lifetime.progress();

        let emitter = q_particle_emitter_state.get(parent).unwrap();

        let color = particle.birth_color * emitter.color.sample_clamped(life);

        let Some(mut material) = res_particle_material_dynamic.get_mut(material_handle.0.id())
        else {
            continue;
        };
        if material.kind != ParticleRenderKind::SkinnedMesh {
            continue;
        }

        let current_uv_offset: Vec2 =
            particle.birth_uv_offset + particle.birth_uv_scroll_rate * lifetime.elapsed_secs();

        material.set_param(
            "vParticleUVTransform",
            uv_transform_rows(Vec2::ONE, current_uv_offset),
        );
        material.set_param("COLOR_LOOKUP_UV", vec2(life, life));
        material.set_param("kColorFactor", color);

        // bones：map.ron 反射出的 SkinnedMeshParticleVs cbuffer（$Globals + PerFrameVertexCB）
        // 不含骨骼矩阵成员，无法经 cbuffer 上传（按名写入即 no-op），
        // 因此跳过逐骨骼矩阵计算；旧静态材质的 bones 槽位与该 SPIR-V 布局本就不匹配。
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
