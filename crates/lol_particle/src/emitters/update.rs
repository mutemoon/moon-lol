//! 统一发射器更新：所有几何族共用同一套发射循环。
//!
//! 因为材质已统一为 [`ParticleMaterialDynamic`]（shader 家族 / 贴图绑定名 / blend
//! 全部由 `kind + emitter_def` 内部推导），所以历史上按材质拆分的五个
//! `update_emitter_*` 系统只剩「附加视觉」一步有差异；这里合并为单个
//! [`update_emitters`] 系统：发射量计算 → 出生参数采样 → 粒子实体 spawn 走公共
//! 骨架，最后按 [`EmitterType`] 分支调用各族的 `attach_*_visuals`。
//!
//! 各族仅存的差异：
//!   - Quad：ParticleMeshQuad 网格 + texture_mult 贴图；
//!   - Mesh：ResourceCache 加载 .scb 网格；
//!   - Decal：ParticleDecal 标记（几何由 update_decal_intersections 后置生成）
//!     + 创建期一次性 uniform；
//!   - Distortion：ParticleMeshDistortion 网格 + normal_map / back-buffer 贴图
//!     + 一次性 uniform + RenderLayers(1)；
//!   - SkinnedMesh：材质覆盖 + spawn_shadow_skin_entity 复制骨骼网格。

use std::f32::consts::PI;
use std::sync::Arc;

use bevy::animation::AnimationTargetId;
use bevy::camera::visibility::RenderLayers;
use bevy::ecs::system::SystemParam;
use bevy::mesh::VertexAttributeValues;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use lol_base_render::camera::{CameraState, TargetImage};
use lol_base_render::mesh_shadow::spawn_shadow_skin_entity;
use lol_base_render::particle::{
    ConfigVfxEmitterDefinition, ConfigVfxPrimitive, ConfigVfxSystemDefinition,
};
use lol_base_render::shader::ShaderMap;
use lol_core::lifetime::Lifetime;

use super::decal::ParticleDecal;
use super::state::{EmitterOf, ParticleEmitterState};
use super::utils::{
    EmissionParams, EmitterType, calculate_emission_params, calculate_particle_transform_frame,
    get_emitter_type,
};
use crate::particle::ParticleState;
use crate::particle::dynamic::{
    ParticleMaterialDynamic, ParticleRenderKind, ParticleTextureInputs,
};
use crate::utils::{ResourceCache, create_black_pixel_texture};
use crate::{
    ATTRIBUTE_LIFETIME, ATTRIBUTE_UV_FRAME, ATTRIBUTE_UV_MULT, ATTRIBUTE_WORLD_POSITION,
    ATTRIBUTE_WORLD_POSITION_VEC4, ParticleId,
};

struct ParticleQuadMeshParams {
    rotation_z: f32,
    frame: f32,
    color: Vec4,
    is_distortion: bool,
}

fn build_particle_quad_mesh(params: ParticleQuadMeshParams) -> Mesh {
    let mut mesh: Mesh = Plane3d::new(Vec3::NEG_Z, Vec2::splat(1.0)).into();

    let transform = Transform::from_rotation(Quat::from_rotation_z(params.rotation_z));

    if let VertexAttributeValues::Float32x3(values) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
    {
        let transformed = values
            .into_iter()
            .map(|v| transform.transform_point(Vec3::from_array(*v)))
            .collect::<Vec<_>>();

        if params.is_distortion {
            let values = transformed
                .iter()
                .map(|v| v.extend(0.0).to_array())
                .collect::<Vec<_>>();
            mesh.insert_attribute(ATTRIBUTE_WORLD_POSITION_VEC4, values);
        } else {
            let values = transformed.iter().map(|v| v.to_array()).collect::<Vec<_>>();
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, values.clone());
            mesh.insert_attribute(ATTRIBUTE_WORLD_POSITION, values);
        }
    }

    if let VertexAttributeValues::Float32x2(values) =
        mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap().clone()
    {
        mesh.insert_attribute(ATTRIBUTE_UV_MULT, values.clone());

        let values = values
            .into_iter()
            .map(|v| {
                if params.is_distortion {
                    [1. - v[0], 1. - v[1], params.frame, params.frame]
                } else {
                    [v[0], v[1], params.frame, 0.0]
                }
            })
            .collect::<Vec<_>>();

        mesh.insert_attribute(ATTRIBUTE_UV_FRAME, values);
    }

    let values = Vec::from([[0.0; 2]; 4]);
    mesh.insert_attribute(ATTRIBUTE_LIFETIME, values);

    let values = Vec::from(
        [[
            params.color.z,
            params.color.y,
            params.color.x,
            params.color.w,
        ]; 4],
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, values);

    mesh
}

/// 皮肤网格相关查询打包，避免系统参数超过 Bevy 的 16 个上限。
#[derive(SystemParam)]
pub struct SkinMeshQueries<'w, 's> {
    q_mesh3d: Query<'w, 's, &'static Mesh3d>,
    q_skinned_mesh: Query<'w, 's, &'static SkinnedMesh>,
    q_children: Query<'w, 's, &'static Children>,
    q_animation_target: Query<'w, 's, (Entity, &'static Transform, &'static AnimationTargetId)>,
    q_parent: Query<'w, 's, &'static ChildOf>,
}

/// 统一发射器更新系统：取代原先按材质拆分的 quad / mesh / decal /
/// skinned_mesh / distortion 五个系统
pub fn update_emitters(
    mut commands: Commands,
    res_assets_vfx_system_definition_data: Res<Assets<ConfigVfxSystemDefinition>>,
    res_asset_server: Res<AssetServer>,
    mut res_mesh: ResMut<Assets<Mesh>>,
    mut res_images: ResMut<Assets<Image>>,
    mut res_resource_cache: ResMut<ResourceCache>,
    mut res_dynamic_material: ResMut<Assets<ParticleMaterialDynamic>>,
    res_shader_map: Option<Res<ShaderMap>>,
    res_target_image: Option<Res<TargetImage>>,
    res_metrics: Option<Res<crate::metrics::ParticleMetricsShared>>,
    mut black_texture_cache: Local<Option<Handle<Image>>>,
    mut query: Query<(
        Entity,
        &EmitterOf,
        &mut Lifetime,
        &mut ParticleEmitterState,
        &ParticleId,
    )>,
    skin_mesh_queries: SkinMeshQueries,
    q_camera: Query<(&Projection, &GlobalTransform), With<CameraState>>,
    time: Res<Time>,
) {
    let Some(shader_map) = res_shader_map.as_deref() else {
        return;
    };

    let cam_data = (|| {
        let (projection, gtransform) = q_camera.single().ok()?;
        let clip_from_view: Mat4 = projection.get_clip_from_view();
        let view_from_world = gtransform.to_matrix().inverse();
        let clip_from_world = clip_from_view * view_from_world;
        let cam_pos = gtransform.translation();
        Some((clip_from_world, cam_pos))
    })();

    // PARTICLE_COLOR_TEXTURE 与 PIXEL_COLOR_REMAP_RAMP 缺省绑定：1×1 黑色贴图（全局缓存一次）。
    let black_texture = black_texture_cache
        .get_or_insert_with(|| res_images.add(create_black_pixel_texture()))
        .clone();
    let color_remap_ramp = black_texture.clone();

    for (emitter_entity, emitter_of, mut lifetime, mut emitter, particle_id) in query.iter_mut() {
        let vfx_emitter_definition_data =
            particle_id.get_def(&res_assets_vfx_system_definition_data);

        let emitter_type = get_emitter_type(vfx_emitter_definition_data);
        if emitter_type == EmitterType::Unknown {
            continue;
        }

        let primitive = vfx_emitter_definition_data
            .primitive
            .clone()
            .unwrap_or(ConfigVfxPrimitive::VfxPrimitiveCameraUnitQuad);

        let Some(EmissionParams {
            particles_to_spawn,
            progress,
        }) = calculate_emission_params(
            &lifetime,
            &mut emitter,
            vfx_emitter_definition_data,
            time.delta_secs(),
        )
        else {
            continue;
        };

        let is_single_particle = vfx_emitter_definition_data
            .is_single_particle
            .unwrap_or(false);
        if is_single_particle {
            lifetime.dead();
        }

        let is_uniform_scale = vfx_emitter_definition_data
            .is_uniform_scale
            .unwrap_or(false);

        let texture = vfx_emitter_definition_data
            .texture
            .as_ref()
            .map(|t| t.handle.clone());
        let particle_color_texture = vfx_emitter_definition_data
            .particle_color_texture
            .as_ref()
            .map(|t| t.handle.clone());
        let texture_mult = vfx_emitter_definition_data
            .texture_mult
            .as_ref()
            .and_then(|tm| tm.texture_mult.as_ref())
            .map(|t| t.handle.clone());
        let erosion_map = vfx_emitter_definition_data
            .alpha_erosion_definition
            .as_ref()
            .and_then(|e| e.erosion_map.as_ref())
            .map(|t| t.handle.clone());

        if particles_to_spawn > 0 {
            let shared_quad_material = if emitter_type == EmitterType::Quad {
                if let Some(h) = &emitter.cached_material {
                    Some(h.clone())
                } else {
                    let emitter_def = Arc::new(vfx_emitter_definition_data.clone());
                    let mut material = ParticleMaterialDynamic::create(
                        ParticleRenderKind::Quad,
                        emitter_def,
                        ParticleTextureInputs {
                            texture: texture.clone(),
                            particle_color_texture: particle_color_texture.clone(),
                            texture_mult: texture_mult.clone(),
                            color_remap_ramp: Some(color_remap_ramp.clone()),
                            erosion_map: erosion_map.clone(),
                            ..default()
                        },
                        shader_map,
                    );
                    if let Some((clip_from_world, cam_pos)) = cam_data.as_ref() {
                        material.set_param("mProj", clip_from_world.transpose());
                        material.set_param("vCamera", *cam_pos);
                    }
                    let handle = res_dynamic_material.add(material);
                    emitter.cached_material = Some(handle.clone());
                    Some(handle)
                }
            } else {
                None
            };

            let shared_distortion_material = if emitter_type == EmitterType::Distortion {
                if let (Some(distortion_definition), Some(res_target_image)) = (
                    vfx_emitter_definition_data.distortion_definition.as_ref(),
                    res_target_image.as_ref(),
                ) {
                    if let Some(h) = &emitter.cached_material {
                        Some(h.clone())
                    } else {
                        let normal_map = distortion_definition
                            .normal_map_texture
                            .as_ref()
                            .map(|t| t.handle.clone());
                        let emitter_def = Arc::new(vfx_emitter_definition_data.clone());
                        let mut material = ParticleMaterialDynamic::create(
                            ParticleRenderKind::Distortion,
                            emitter_def,
                            ParticleTextureInputs {
                                texture: texture.clone(),
                                particle_color_texture: particle_color_texture.clone(),
                                normal_map,
                                back_buffer: Some(res_target_image.0.clone()),
                                erosion_map: erosion_map.clone(),
                                ..default()
                            },
                            shader_map,
                        );
                        if let Some((clip_from_world, cam_pos)) = cam_data.as_ref() {
                            material.set_param("mProj", clip_from_world.transpose());
                            material.set_param("vCamera", *cam_pos);
                        }
                        material.set_param("PARTICLE_DEPTH_PUSH_PULL", 0.0f32);
                        material.set_param(
                            "AlphaTestReferenceValue",
                            vfx_emitter_definition_data.alpha_ref.unwrap_or(0) as f32,
                        );
                        material.set_param(
                            "DistortionPower",
                            distortion_definition.distortion.unwrap_or(1.0),
                        );
                        let handle = res_dynamic_material.add(material);
                        emitter.cached_material = Some(handle.clone());
                        Some(handle)
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let shared_decal_material = if emitter_type == EmitterType::Decal {
                if let ConfigVfxPrimitive::VfxPrimitivePlanarProjection {
                    y_range: Some(y_range),
                } = &primitive
                {
                    if let Some(h) = &emitter.cached_material {
                        Some(h.clone())
                    } else {
                        let emitter_def = Arc::new(vfx_emitter_definition_data.clone());
                        let mut material = ParticleMaterialDynamic::create(
                            ParticleRenderKind::UnlitDecal,
                            emitter_def,
                            ParticleTextureInputs {
                                texture: texture.clone(),
                                particle_color_texture: particle_color_texture.clone(),
                                erosion_map: erosion_map.clone(),
                                ..default()
                            },
                            shader_map,
                        );
                        if let Some((clip_from_world, cam_pos)) = cam_data.as_ref() {
                            material.set_param("mProj", clip_from_world.transpose());
                            material.set_param("vCamera", *cam_pos);
                        }
                        material.set_param("DECAL_PROJECTION_Y_RANGE", Vec4::splat(*y_range));
                        material.set_param("DECAL_WORLD_MATRIX", Mat4::IDENTITY);
                        material.set_param(
                            "DECAL_WORLD_TO_UV_MATRIX",
                            (Mat4::from_translation(Vec3::splat(0.5))
                                * emitter.global_transform.to_matrix().inverse())
                            .transpose(),
                        );
                        material.set_param("MODULATE_COLOR", Vec4::ONE);
                        material.set_param("COLOR_UV", Vec2::ONE);
                        let handle = res_dynamic_material.add(material);
                        emitter.cached_material = Some(handle.clone());
                        Some(handle)
                    }
                } else {
                    None
                }
            } else {
                None
            };

            for _ in 0..particles_to_spawn {
                let particle_lifetime = emitter.particle_lifetime.sample_clamped(progress);
                let particle_lifetime = if particle_lifetime < 0. {
                    0.
                } else {
                    particle_lifetime
                };

                let raw_rotation0 = emitter.birth_rotation0.sample_clamped(progress);
                let raw_scale0 = emitter.birth_scale0.sample_clamped(progress);

                let (transform, shape_rotation, adjusted_birth_scale0, frame) =
                    calculate_particle_transform_frame(
                        raw_rotation0,
                        raw_scale0,
                        is_uniform_scale,
                        vfx_emitter_definition_data,
                        &primitive,
                        progress,
                    );

                let raw_velocity = emitter.birth_velocity.sample_clamped(progress);
                let velocity = shape_rotation * raw_velocity;

                let birth_color = emitter.birth_color.sample_clamped(progress);

                let particle_state = ParticleState {
                    birth_uv_offset: emitter.birth_uv_offset.sample_clamped(progress),
                    birth_uv_scroll_rate: emitter.birth_uv_scroll_rate.sample_clamped(progress),
                    birth_color,
                    birth_scale0: adjusted_birth_scale0,
                    initial_rotation: transform.rotation,
                    velocity,
                    acceleration: emitter.birth_acceleration.sample_clamped(progress),
                    frame,
                };

                let particle_entity = commands
                    .spawn((
                        particle_id.clone(),
                        particle_state,
                        Lifetime::new_timer(particle_lifetime),
                        transform,
                        Pickable::IGNORE,
                        ChildOf(emitter_entity),
                    ))
                    .id();

                if let Some(metrics) = res_metrics.as_ref() {
                    metrics
                        .particles_spawned
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }

                match emitter_type {
                    EmitterType::Quad => {
                        if let Some(mat_handle) = &shared_quad_material {
                            attach_quad_visuals(
                                &mut commands,
                                particle_entity,
                                frame,
                                birth_color,
                                mat_handle.clone(),
                                &mut res_mesh,
                            );
                        }
                    }
                    EmitterType::Mesh => {
                        let simple_mesh_name = match &primitive {
                            ConfigVfxPrimitive::VfxPrimitiveMesh {
                                simple_mesh_name, ..
                            } => simple_mesh_name.as_deref(),
                            _ => None,
                        };
                        attach_mesh_visuals(
                            &mut commands,
                            particle_entity,
                            vfx_emitter_definition_data,
                            simple_mesh_name,
                            texture.clone(),
                            particle_color_texture.clone(),
                            erosion_map.clone(),
                            Some(color_remap_ramp.clone()),
                            &mut res_dynamic_material,
                            &mut res_resource_cache,
                            &res_asset_server,
                            shader_map,
                            cam_data.as_ref(),
                        );
                    }
                    EmitterType::Decal => {
                        if let Some(mat_handle) = &shared_decal_material {
                            attach_unlit_decal_visuals(
                                &mut commands,
                                particle_entity,
                                mat_handle.clone(),
                            );
                        }
                    }
                    EmitterType::Distortion => {
                        if let Some(mat_handle) = &shared_distortion_material {
                            attach_distortion_visuals(
                                &mut commands,
                                particle_entity,
                                frame,
                                birth_color,
                                mat_handle.clone(),
                                &mut res_mesh,
                            );
                        }
                    }
                    EmitterType::SkinnedMesh => attach_skinned_mesh_visuals(
                        &mut commands,
                        particle_entity,
                        emitter_of,
                        vfx_emitter_definition_data,
                        texture.clone(),
                        particle_color_texture.clone(),
                        erosion_map.clone(),
                        Some(color_remap_ramp.clone()),
                        &mut res_dynamic_material,
                        shader_map,
                        cam_data.as_ref(),
                        &skin_mesh_queries.q_mesh3d,
                        &skin_mesh_queries.q_skinned_mesh,
                        &skin_mesh_queries.q_children,
                        &skin_mesh_queries.q_animation_target,
                        &skin_mesh_queries.q_parent,
                    ),
                    EmitterType::Unknown => unreachable!(),
                }
            }
        }
    }
}

pub fn attach_quad_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    frame: f32,
    birth_color: Vec4,
    material_handle: Handle<ParticleMaterialDynamic>,
    res_mesh: &mut ResMut<Assets<Mesh>>,
) {
    let mesh = res_mesh.add(build_particle_quad_mesh(ParticleQuadMeshParams {
        rotation_z: PI / 2.,
        frame,
        color: birth_color,
        is_distortion: false,
    }));
    commands
        .entity(particle_entity)
        .insert((Mesh3d(mesh), MeshMaterial3d(material_handle)));
}

pub fn attach_mesh_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    vfx_emitter_definition_data: &ConfigVfxEmitterDefinition,
    mesh_name: Option<&str>,
    texture: Option<Handle<Image>>,
    particle_color_texture: Option<Handle<Image>>,
    erosion_map: Option<Handle<Image>>,
    color_remap_ramp: Option<Handle<Image>>,
    res_dynamic_material: &mut ResMut<Assets<ParticleMaterialDynamic>>,
    res_resource_cache: &mut ResMut<ResourceCache>,
    res_asset_server: &Res<AssetServer>,
    shader_map: &ShaderMap,
    cam_data: Option<&(Mat4, Vec3)>,
) {
    let Some(mesh_name) = mesh_name else {
        println!("VfxPrimitiveMesh: mesh_name is None");
        return;
    };

    let mesh = res_resource_cache.get_mesh(res_asset_server, mesh_name);

    // blend_mode 由 emitter_def 内部推导；mWorld / UV 变换等逐帧参数在
    // update_particle 的 Mesh 分支按成员名写入
    let emitter_def = Arc::new(vfx_emitter_definition_data.clone());
    let mut material = ParticleMaterialDynamic::create(
        ParticleRenderKind::Mesh,
        emitter_def,
        ParticleTextureInputs {
            texture,
            particle_color_texture,
            color_remap_ramp,
            erosion_map,
            ..default()
        },
        shader_map,
    );
    if let Some((clip_from_world, cam_pos)) = cam_data {
        material.set_param("mProj", clip_from_world.transpose());
        material.set_param("vCamera", *cam_pos);
    }

    commands.entity(particle_entity).insert((
        Mesh3d(mesh),
        MeshMaterial3d(res_dynamic_material.add(material)),
    ));
}

pub fn attach_unlit_decal_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    material_handle: Handle<ParticleMaterialDynamic>,
) {
    commands
        .entity(particle_entity)
        .insert((ParticleDecal::default(), MeshMaterial3d(material_handle)));
}

pub fn attach_distortion_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    frame: f32,
    birth_color: Vec4,
    material_handle: Handle<ParticleMaterialDynamic>,
    res_mesh: &mut ResMut<Assets<Mesh>>,
) {
    let mesh = res_mesh.add(build_particle_quad_mesh(ParticleQuadMeshParams {
        rotation_z: -PI / 2.,
        frame,
        color: birth_color,
        is_distortion: true,
    }));
    commands.entity(particle_entity).insert((
        Mesh3d(mesh),
        MeshMaterial3d(material_handle),
        RenderLayers::layer(1),
    ));
}

pub fn attach_skinned_mesh_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    emitter_of: &EmitterOf,
    vfx_emitter_definition_data: &ConfigVfxEmitterDefinition,
    texture: Option<Handle<Image>>,
    particle_color_texture: Option<Handle<Image>>,
    erosion_map: Option<Handle<Image>>,
    color_remap_ramp: Option<Handle<Image>>,
    res_dynamic_material: &mut ResMut<Assets<ParticleMaterialDynamic>>,
    shader_map: &ShaderMap,
    cam_data: Option<&(Mat4, Vec3)>,
    q_mesh3d: &Query<&Mesh3d>,
    q_skinned_mesh: &Query<&SkinnedMesh>,
    q_children: &Query<&Children>,
    q_animation_target: &Query<(Entity, &Transform, &AnimationTargetId)>,
    q_parent: &Query<&ChildOf>,
) {
    let final_texture = if let Some(material_override_definitions) =
        &vfx_emitter_definition_data.material_override_definitions
    {
        let mut tex = texture;
        for material_override_definition in material_override_definitions {
            if let Some(base_texture) = &material_override_definition.base_texture {
                tex = Some(base_texture.handle.clone());
            }
        }
        tex
    } else {
        texture
    };

    let emitter_def = Arc::new(vfx_emitter_definition_data.clone());
    let mut material = ParticleMaterialDynamic::create(
        ParticleRenderKind::SkinnedMesh,
        emitter_def,
        ParticleTextureInputs {
            texture: final_texture,
            particle_color_texture,
            color_remap_ramp,
            erosion_map,
            ..default()
        },
        shader_map,
    );
    if let Some((clip_from_world, cam_pos)) = cam_data {
        material.set_param("mProj", clip_from_world.transpose());
        material.set_param("vCamera", *cam_pos);
    }
    let material_handle = MeshMaterial3d(res_dynamic_material.add(material));

    spawn_shadow_skin_entity(
        commands,
        particle_entity,
        emitter_of.0,
        material_handle,
        q_mesh3d,
        q_skinned_mesh,
        q_children,
        q_animation_target,
        q_parent,
    );
}
