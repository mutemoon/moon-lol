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

use std::sync::Arc;

use bevy::animation::AnimationTargetId;
use bevy::camera::visibility::RenderLayers;
use bevy::ecs::system::SystemParam;
use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use lol_base_render::camera::TargetImage;
use lol_base_render::mesh_shadow::spawn_shadow_skin_entity;
use lol_base_render::particle::{
    ConfigVfxDistortionDefinition, ConfigVfxEmitterDefinition, ConfigVfxPrimitive,
    ConfigVfxSystemDefinition,
};
use lol_base_render::shader::ShaderMap;
use lol_core::lifetime::Lifetime;

use super::decal::ParticleDecal;
use super::state::{EmitterOf, ParticleEmitterState};
use super::utils::{
    EmissionParams, EmitterType, ParticleBirthParams, calculate_emission_params,
    calculate_particle_transform_frame, get_emitter_type, spawn_particle_entity,
};
use crate::ParticleId;
use crate::particle::distortion::ParticleMeshDistortion;
use crate::particle::dynamic::{
    ParticleMaterialDynamic, ParticleRenderKind, ParticleTextureInputs,
};
use crate::particle::quad::ParticleMeshQuad;
use crate::utils::{ResourceCache, create_black_pixel_texture};

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
    mut black_texture_cache: Local<Option<Handle<Image>>>,
    mut query: Query<(
        Entity,
        &EmitterOf,
        &mut Lifetime,
        &mut ParticleEmitterState,
        &ParticleId,
    )>,
    skin_mesh_queries: SkinMeshQueries,
    time: Res<Time>,
) {
    let Some(shader_map) = res_shader_map.as_deref() else {
        return;
    };

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

        for _ in 0..particles_to_spawn {
            let particle_lifetime = emitter.particle_lifetime.sample_clamped(progress);
            let particle_lifetime = if particle_lifetime < 0. {
                0.
            } else {
                particle_lifetime
            };

            let birth_params = ParticleBirthParams::sample(&mut emitter, progress);

            let (transform, adjusted_birth_scale0, frame) = calculate_particle_transform_frame(
                &birth_params,
                is_uniform_scale,
                vfx_emitter_definition_data,
                &primitive,
                progress,
            );

            let particle_entity = spawn_particle_entity(
                &mut commands,
                particle_id,
                emitter_entity,
                particle_lifetime,
                transform,
                frame,
                &birth_params,
                adjusted_birth_scale0,
            );

            match emitter_type {
                EmitterType::Quad => attach_quad_visuals(
                    &mut commands,
                    particle_entity,
                    vfx_emitter_definition_data,
                    frame,
                    texture.clone(),
                    particle_color_texture.clone(),
                    texture_mult.clone(),
                    erosion_map.clone(),
                    Some(color_remap_ramp.clone()),
                    &mut res_mesh,
                    &mut res_dynamic_material,
                    shader_map,
                ),
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
                    );
                }
                EmitterType::Decal => {
                    if let ConfigVfxPrimitive::VfxPrimitivePlanarProjection {
                        y_range: Some(y_range),
                    } = &primitive
                    {
                        attach_unlit_decal_visuals(
                            &mut commands,
                            particle_entity,
                            vfx_emitter_definition_data,
                            *y_range,
                            texture.clone(),
                            particle_color_texture.clone(),
                            erosion_map.clone(),
                            &mut res_dynamic_material,
                            shader_map,
                        );
                    }
                }
                EmitterType::Distortion => {
                    let (Some(distortion_definition), Some(res_target_image)) = (
                        vfx_emitter_definition_data.distortion_definition.as_ref(),
                        res_target_image.as_ref(),
                    ) else {
                        continue;
                    };
                    attach_distortion_visuals(
                        &mut commands,
                        particle_entity,
                        vfx_emitter_definition_data,
                        distortion_definition,
                        texture.clone(),
                        particle_color_texture.clone(),
                        erosion_map.clone(),
                        frame,
                        &mut res_mesh,
                        &mut res_dynamic_material,
                        res_target_image,
                        shader_map,
                    );
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

pub fn attach_quad_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    vfx_emitter_definition_data: &ConfigVfxEmitterDefinition,
    frame: f32,
    texture: Option<Handle<Image>>,
    particle_color_texture: Option<Handle<Image>>,
    texture_mult: Option<Handle<Image>>,
    erosion_map: Option<Handle<Image>>,
    color_remap_ramp: Option<Handle<Image>>,
    res_mesh: &mut ResMut<Assets<Mesh>>,
    res_dynamic_material: &mut ResMut<Assets<ParticleMaterialDynamic>>,
    shader_map: &ShaderMap,
) {
    let mesh = res_mesh.add(ParticleMeshQuad { frame });
    commands.entity(particle_entity).insert(Mesh3d(mesh));

    // blend_mode / slice 家族由 emitter_def 内部推导；各贴图在发射器处解析后传入
    let emitter_def = Arc::new(vfx_emitter_definition_data.clone());
    let material = ParticleMaterialDynamic::create(
        ParticleRenderKind::Quad,
        emitter_def,
        ParticleTextureInputs {
            texture,
            particle_color_texture,
            texture_mult,
            color_remap_ramp,
            erosion_map,
            ..default()
        },
        shader_map,
    );
    commands
        .entity(particle_entity)
        .insert(MeshMaterial3d(res_dynamic_material.add(material)));
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
) {
    let Some(mesh_name) = mesh_name else {
        println!("VfxPrimitiveMesh: mesh_name is None");
        return;
    };

    let mesh = res_resource_cache.get_mesh(res_asset_server, mesh_name);

    // blend_mode 由 emitter_def 内部推导；mWorld / UV 变换等逐帧参数在
    // update_particle 的 Mesh 分支按成员名写入
    let emitter_def = Arc::new(vfx_emitter_definition_data.clone());
    let material = ParticleMaterialDynamic::create(
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

    commands.entity(particle_entity).insert((
        Mesh3d(mesh),
        MeshMaterial3d(res_dynamic_material.add(material)),
    ));
}

pub fn attach_unlit_decal_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    vfx_emitter_definition_data: &ConfigVfxEmitterDefinition,
    y_range: f32,
    texture: Option<Handle<Image>>,
    particle_color_texture: Option<Handle<Image>>,
    erosion_map: Option<Handle<Image>>,
    res_dynamic_material: &mut ResMut<Assets<ParticleMaterialDynamic>>,
    shader_map: &ShaderMap,
) {
    // blend_mode 由 emitter_def 内部推导；世界→UV 矩阵逐帧在 update_particle
    // 的 UnlitDecal 分支写入
    let emitter_def = Arc::new(vfx_emitter_definition_data.clone());
    let mut material = ParticleMaterialDynamic::create(
        ParticleRenderKind::UnlitDecal,
        emitter_def,
        ParticleTextureInputs {
            texture,
            particle_color_texture,
            erosion_map,
            ..default()
        },
        shader_map,
    );

    // 创建期一次性参数（成员名已按 UnlitDecalVs/Ps 的 unified 布局核实）：
    // ParticleDecalVS 的投影 Y 范围与世界矩阵；$Globals 的调制色/颜色 UV
    //（对齐旧静态材质默认值，避免零填充 blob 乘黑输出）
    material.set_param("DECAL_PROJECTION_Y_RANGE", Vec4::splat(y_range));
    material.set_param("DECAL_WORLD_MATRIX", Mat4::IDENTITY);
    material.set_param("MODULATE_COLOR", Vec4::ONE);
    material.set_param("COLOR_UV", Vec2::ONE);

    commands.entity(particle_entity).insert((
        ParticleDecal::default(),
        MeshMaterial3d(res_dynamic_material.add(material)),
    ));
}

pub fn attach_distortion_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    vfx_emitter_definition_data: &ConfigVfxEmitterDefinition,
    distortion_definition: &ConfigVfxDistortionDefinition,
    texture: Option<Handle<Image>>,
    particle_color_texture: Option<Handle<Image>>,
    erosion_map: Option<Handle<Image>>,
    frame: f32,
    res_mesh: &mut ResMut<Assets<Mesh>>,
    res_dynamic_material: &mut ResMut<Assets<ParticleMaterialDynamic>>,
    res_target_image: &Res<TargetImage>,
    shader_map: &ShaderMap,
) {
    let mesh = res_mesh.add(ParticleMeshDistortion { frame });
    commands.entity(particle_entity).insert(Mesh3d(mesh));

    // 法线扰动贴图：VfxTexture 已由 loader 解析出 handle，取 handle 克隆使用
    let normal_map = distortion_definition
        .normal_map_texture
        .as_ref()
        .map(|t| t.handle.clone());

    // blend_mode 由 emitter_def 内部推导；back-buffer 拷贝绑到扭曲采样槽
    let emitter_def = Arc::new(vfx_emitter_definition_data.clone());
    let mut material = ParticleMaterialDynamic::create(
        ParticleRenderKind::Distortion,
        emitter_def,
        ParticleTextureInputs {
            texture,
            particle_color_texture,
            normal_map,
            back_buffer: Some(res_target_image.0.clone()),
            erosion_map,
            ..default()
        },
        shader_map,
    );

    // 创建期一次性参数（成员名已按 DistortionVs/Ps 的 unified 布局核实）
    let texture_info = match vfx_emitter_definition_data.tex_div {
        Some(tex_div) => vec4(tex_div.x, 1.0 / tex_div.x, 1.0 / tex_div.y, 0.),
        None => Vec4::ONE,
    };
    material.set_param("TEXTURE_INFO", texture_info);
    material.set_param("PARTICLE_DEPTH_PUSH_PULL", 0.0f32);
    // unified 并集里 AlphaTestReferenceValue 与 DistortionPower 同居 offset 0
    //（各变体独占其一），后写 DistortionPower 使扭曲强度生效
    material.set_param(
        "AlphaTestReferenceValue",
        vfx_emitter_definition_data.alpha_ref.unwrap_or(0) as f32,
    );
    material.set_param(
        "DistortionPower",
        distortion_definition.distortion.unwrap_or(1.0),
    );

    commands.entity(particle_entity).insert((
        MeshMaterial3d(res_dynamic_material.add(material)),
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
    let material = MeshMaterial3d(res_dynamic_material.add(ParticleMaterialDynamic::create(
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
    )));

    spawn_shadow_skin_entity(
        commands,
        particle_entity,
        emitter_of.0,
        material,
        q_mesh3d,
        q_skinned_mesh,
        q_children,
        q_animation_target,
        q_parent,
    );
}
