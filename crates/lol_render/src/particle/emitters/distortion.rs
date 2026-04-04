use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use league_core::extract::{
    EnumVfxPrimitive, VfxDistortionDefinitionData, VfxEmitterDefinitionData,
    VfxSystemDefinitionData,
};
use lol_core::lifetime::Lifetime;

use super::state::ParticleEmitterState;
use super::utils::{
    calculate_emission_params, calculate_particle_transform_frame, get_emitter_type,
    spawn_particle_entity, EmissionParams, EmitterType, ParticleBirthParams,
};
use crate::camera::TargetImage;
use crate::particle::particle::distortion::{
    ParticleMaterialDistortion, ParticleMeshDistortion, UniformsPixelDistortion,
    UniformsVertexDistortion,
};
use crate::particle::ParticleId;
use crate::resource::ResourceCache;

pub fn attach_distortion_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    vfx_emitter_definition_data: &VfxEmitterDefinitionData,
    distortion_definition: &VfxDistortionDefinitionData,
    texture: Option<Handle<Image>>,
    particle_color_texture: Option<Handle<Image>>,
    blend_mode: u8,
    frame: f32,
    res_mesh: &mut ResMut<Assets<Mesh>>,
    _res_image: &mut ResMut<Assets<Image>>,
    res_distortion_material: &mut ResMut<Assets<ParticleMaterialDistortion>>,
    res_asset_server: &Res<AssetServer>,
    res_resource_cache: &mut ResMut<ResourceCache>,
    res_target_image: &Res<TargetImage>,
) {
    info!(
        "[扭曲效果] 开始附加扭曲视觉效果到粒子 {:?}",
        particle_entity
    );

    let mesh = res_mesh.add(ParticleMeshDistortion { frame });
    commands.entity(particle_entity).insert(Mesh3d(mesh));
    info!("[扭曲效果] 已添加网格，帧数: {}", frame);

    // Load normal map from distortion definition
    info!("[扭曲效果] 正在加载法线贴图...");
    let normal_map = distortion_definition.normal_map_texture.as_ref().map(|v| {
        info!("[扭曲效果] 法线贴图路径: {}", v);
        res_resource_cache.get_image(&res_asset_server, v)
    });

    let distortion_power = distortion_definition.distortion.unwrap_or(1.0);
    info!("[扭曲效果] 扭曲强度: {}", distortion_power);

    let texture_info = match vfx_emitter_definition_data.tex_div {
        Some(tex_div) => {
            info!(
                "[扭曲效果] 纹理分割 - 列数: {}, 行数: {}",
                tex_div.x, tex_div.y
            );
            vec4(tex_div.x, 1.0 / tex_div.x, 1.0 / tex_div.y, 0.)
        }
        None => Vec4::ONE,
    };

    let uniforms_vertex = UniformsVertexDistortion {
        particle_depth_push_pull: 0.0,
        texture_info,
    };

    let uniforms_pixel = UniformsPixelDistortion {
        alpha_test_reference_value: vfx_emitter_definition_data.alpha_ref.unwrap_or(0) as f32,
        distortion_power,
        apply_team_color_correction: Vec4::ZERO,
    };

    info!("[扭曲效果] 混合模式: {}", blend_mode);
    info!("[扭曲效果] 纹理: {:?}", texture.is_some());
    info!(
        "[扭曲效果] 粒子颜色纹理: {:?}",
        particle_color_texture.is_some()
    );

    commands.entity(particle_entity).insert((
        MeshMaterial3d(res_distortion_material.add(ParticleMaterialDistortion {
            uniforms_vertex,
            uniforms_pixel,
            texture: texture.clone(),
            particle_color_texture: particle_color_texture.clone(),
            normal_map,
            cmb_tex_sampler_back_buffer_copy_smp_clamp_no_mip: Some(res_target_image.0.clone()),
            blend_mode,
        })),
        RenderLayers::layer(1),
    ));

    info!("[扭曲效果] 扭曲材质已附加完成");
}

/// Update emitters with distortion definition
pub fn update_emitter_distortion(
    mut commands: Commands,
    mut res_mesh: ResMut<Assets<Mesh>>,
    res_assets_vfx_system_definition_data: Res<Assets<VfxSystemDefinitionData>>,
    res_asset_server: Res<AssetServer>,
    mut res_resource_cache: ResMut<ResourceCache>,
    mut res_image: ResMut<Assets<Image>>,
    mut res_distortion_material: ResMut<Assets<ParticleMaterialDistortion>>,
    res_target_image: Res<TargetImage>,
    mut query: Query<(
        Entity,
        &mut Lifetime,
        &mut ParticleEmitterState,
        &ParticleId,
    )>,
    time: Res<Time>,
) {
    for (emitter_entity, mut lifetime, mut emitter, particle_id) in query.iter_mut() {
        let vfx_emitter_definition_data =
            particle_id.get_def(&res_assets_vfx_system_definition_data);

        // Check if this emitter should be processed by this update function
        let emitter_type = get_emitter_type(vfx_emitter_definition_data);
        if emitter_type != EmitterType::Distortion {
            continue;
        }

        // let vert_path = get_shader_handle(ParticleMaterialDistortion::VERT_PATH, &vec![]);
        // let frag_path = get_shader_handle(ParticleMaterialDistortion::FRAG_PATH, &vec![]);
        // let vert = res_shader.get(&vert_path).unwrap();
        // let frag = res_shader.get(&frag_path).unwrap();
        // info!("[扭曲发射器] 着色器: {}", vert.source.as_str());
        // info!("[扭曲发射器] 着色器: {}", frag.source.as_str());

        // Check if this emitter has a distortion definition
        let Some(distortion_definition) = &vfx_emitter_definition_data.distortion_definition else {
            continue;
        };

        info!(
            "[扭曲发射器] 发现有扭曲定义的发射器: {:?}",
            vfx_emitter_definition_data.emitter_name
        );
        info!(
            "[扭曲发射器] 扭曲模式: {:?}",
            distortion_definition.distortion_mode
        );

        let primitive = vfx_emitter_definition_data
            .primitive
            .clone()
            .unwrap_or(EnumVfxPrimitive::VfxPrimitiveCameraUnitQuad);

        info!("[扭曲发射器] 图元类型: {:?}", primitive);

        // Distortion typically works with quad-like primitives
        let is_valid_primitive = matches!(
            primitive,
            EnumVfxPrimitive::VfxPrimitiveArbitraryQuad
                | EnumVfxPrimitive::VfxPrimitiveCameraUnitQuad
        );
        if !is_valid_primitive {
            info!("[扭曲发射器] 图元类型不支持扭曲效果，跳过");
            continue;
        }

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

        info!("[扭曲发射器] 需要生成的粒子数: {}", particles_to_spawn);
        info!("[扭曲发射器] 发射进度: {:.2}", progress);

        let is_single_particle = vfx_emitter_definition_data
            .is_single_particle
            .unwrap_or(false);
        if is_single_particle {
            info!("[扭曲发射器] 单粒子模式，标记发射器为死亡");
            lifetime.dead();
        }

        let is_uniform_scale = vfx_emitter_definition_data
            .is_uniform_scale
            .unwrap_or(false);

        info!("[扭曲发射器] 均匀缩放: {}", is_uniform_scale);

        let texture = vfx_emitter_definition_data
            .texture
            .as_ref()
            .map(|v| res_resource_cache.get_image_srgb(&res_asset_server, v));

        if let Some(ref tex_path) = vfx_emitter_definition_data.texture {
            info!("[扭曲发射器] 纹理路径: {}", tex_path);
        }

        let particle_color_texture = vfx_emitter_definition_data
            .particle_color_texture
            .as_ref()
            .map(|v| res_resource_cache.get_image(&res_asset_server, v));

        if let Some(ref color_tex_path) = vfx_emitter_definition_data.particle_color_texture {
            info!("[扭曲发射器] 粒子颜色纹理路径: {}", color_tex_path);
        }

        let blend_mode = vfx_emitter_definition_data.blend_mode.unwrap_or(4);

        for i in 0..particles_to_spawn {
            info!("[扭曲发射器] 生成第 {} 个扭曲粒子", i + 1);

            let particle_lifetime = emitter.particle_lifetime.sample_clamped(progress);
            let particle_lifetime = if particle_lifetime < 0. {
                0.
            } else {
                particle_lifetime
            };

            info!("[扭曲发射器] 粒子生命周期: {:.2} 秒", particle_lifetime);

            let birth_params = ParticleBirthParams::sample(&mut emitter, progress);

            let (transform, adjusted_birth_scale0, frame) = calculate_particle_transform_frame(
                &birth_params,
                is_uniform_scale,
                vfx_emitter_definition_data,
                &primitive,
                progress,
            );

            info!("[扭曲发射器] 粒子帧数: {}", frame);
            info!("[扭曲发射器] 粒子位置: {:?}", transform.translation);

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

            info!("[扭曲发射器] 已创建粒子实体 {:?}", particle_entity);

            attach_distortion_visuals(
                &mut commands,
                particle_entity,
                vfx_emitter_definition_data,
                distortion_definition,
                texture.clone(),
                particle_color_texture.clone(),
                blend_mode,
                frame,
                &mut res_mesh,
                &mut res_image,
                &mut res_distortion_material,
                &res_asset_server,
                &mut res_resource_cache,
                &res_target_image,
            );
        }

        info!("[扭曲发射器] 粒子生成完成");
    }
}
