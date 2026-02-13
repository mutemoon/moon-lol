use bevy::prelude::*;
use league_core::{EnumVfxPrimitive, VfxEmitterDefinitionData, VfxSystemDefinitionData};

use crate::{
    create_black_pixel_texture, Lifetime, ParticleId, ParticleMaterialQuad,
    ParticleMaterialQuadSlice, ParticleMeshQuad, ResourceCache, UniformsPixelQuadSlice,
    UniformsVertexQuad,
};

use super::ParticleEmitterState;
use super::utils::{ParticleBirthParams, EmissionParams, calculate_emission_params, calculate_particle_transform_frame, spawn_particle_entity, get_emitter_type, EmitterType};

pub fn attach_quad_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    vfx_emitter_definition_data: &VfxEmitterDefinitionData,
    texture: Option<Handle<Image>>,
    particle_color_texture: Option<Handle<Image>>,
    texture_mult: Option<Handle<Image>>,
    blend_mode: u8,
    frame: f32,
    res_mesh: &mut ResMut<Assets<Mesh>>,
    res_image: &mut ResMut<Assets<Image>>,
    res_quad_material: &mut ResMut<Assets<ParticleMaterialQuad>>,
    res_quad_slice_material: &mut ResMut<Assets<ParticleMaterialQuadSlice>>,
) {
    let mesh = res_mesh.add(ParticleMeshQuad { frame });
    commands.entity(particle_entity).insert(Mesh3d(mesh));

    let black_pixel_texture = res_image.add(create_black_pixel_texture());
    let uniforms_vertex = UniformsVertexQuad {
        texture_info: match vfx_emitter_definition_data.tex_div {
            Some(tex_div) => vec4(tex_div.x, 1.0 / tex_div.x, 1.0 / tex_div.y, 0.),
            None => Vec4::ONE,
        },
        ..default()
    };

    if let Some(range) = vfx_emitter_definition_data.slice_technique_range {
        commands.entity(particle_entity).insert(MeshMaterial3d(
            res_quad_slice_material.add(ParticleMaterialQuadSlice {
                uniforms_vertex,
                uniforms_pixel: UniformsPixelQuadSlice {
                    slice_range: vec2(range, 1.0 / (range * range)),
                    ..default()
                },
                particle_color_texture: particle_color_texture.clone(),
                texture: texture.clone(),
                cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: Some(black_pixel_texture),
                sampler_fow: None,
                blend_mode,
            }),
        ));
    } else {
        commands
            .entity(particle_entity)
            .insert(MeshMaterial3d(res_quad_material.add(ParticleMaterialQuad {
                uniforms_vertex,
                particle_color_texture: particle_color_texture.clone(),
                texture: texture.clone(),
                cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: Some(black_pixel_texture),
                texturemult: texture_mult.clone(),
                blend_mode,
                ..default()
            })));
    };
}

/// Update emitters for Quad primitive types (ArbitraryQuad, CameraUnitQuad)
pub fn update_emitter_quad(
    mut commands: Commands,
    mut res_mesh: ResMut<Assets<Mesh>>,
    res_assets_vfx_system_definition_data: Res<Assets<VfxSystemDefinitionData>>,
    res_asset_server: Res<AssetServer>,
    mut res_resource_cache: ResMut<ResourceCache>,
    mut res_image: ResMut<Assets<Image>>,
    mut res_quad_material: ResMut<Assets<ParticleMaterialQuad>>,
    mut res_quad_slice_material: ResMut<Assets<ParticleMaterialQuadSlice>>,
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
        if emitter_type != EmitterType::Quad {
            continue;
        }

        let primitive = vfx_emitter_definition_data
            .primitive
            .clone()
            .unwrap_or(EnumVfxPrimitive::VfxPrimitiveCameraUnitQuad);

        let Some(EmissionParams {
            particles_to_spawn,
            progress,
        }) = calculate_emission_params(
            &lifetime,
            &mut emitter,
            vfx_emitter_definition_data,
            time.delta_secs(),
        ) else {
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
            .map(|v| res_resource_cache.get_image_srgb(&res_asset_server, v));

        let particle_color_texture = vfx_emitter_definition_data
            .particle_color_texture
            .as_ref()
            .map(|v| res_resource_cache.get_image(&res_asset_server, v));

        let texture_mult = vfx_emitter_definition_data
            .texture_mult
            .as_ref()
            .and_then(|v| v.texture_mult.as_ref())
            .map(|v| res_resource_cache.get_image(&res_asset_server, v));

        let blend_mode = vfx_emitter_definition_data.blend_mode.unwrap_or(4);

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

            attach_quad_visuals(
                &mut commands,
                particle_entity,
                vfx_emitter_definition_data,
                texture.clone(),
                particle_color_texture.clone(),
                texture_mult.clone(),
                blend_mode,
                frame,
                &mut res_mesh,
                &mut res_image,
                &mut res_quad_material,
                &mut res_quad_slice_material,
            );
        }
    }
}
