use bevy::prelude::*;
use league_core::extract::{
    EnumVfxPrimitive, VfxPrimitivePlanarProjection, VfxProjectionDefinitionData,
    VfxSystemDefinitionData,
};
use lol_core::lifetime::Lifetime;
use lol_core_render::resource_cache::ResourceCache;

use super::state::ParticleEmitterState;
use super::utils::{
    calculate_emission_params, calculate_particle_transform_frame, get_emitter_type,
    spawn_particle_entity, EmissionParams, EmitterType, ParticleBirthParams,
};
use crate::emitters::decal::ParticleDecal;
use crate::environment::unlit_decal::{
    ParticleMaterialUnlitDecal, UniformsPixelUnlitDecal, UniformsVertexUnlitDecal,
};
use crate::ParticleId;

pub fn attach_unlit_decal_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    m_projection: &VfxProjectionDefinitionData,
    texture: Option<Handle<Image>>,
    particle_color_texture: Option<Handle<Image>>,
    blend_mode: u8,
    res_unlit_decal_material: &mut ResMut<Assets<ParticleMaterialUnlitDecal>>,
) {
    let material_handle = res_unlit_decal_material.add(ParticleMaterialUnlitDecal {
        uniforms_vertex: UniformsVertexUnlitDecal {
            decal_projection_y_range: Vec4::splat(m_projection.m_y_range.unwrap()),
            ..default()
        },
        uniforms_pixel: UniformsPixelUnlitDecal::default(),
        diffuse_map: texture.clone(),
        particle_color_texture: particle_color_texture.clone(),
        cmb_tex_fow_map_smp_clamp_no_mip: None,
        blend_mode,
    });

    commands
        .entity(particle_entity)
        .insert((ParticleDecal::default(), MeshMaterial3d(material_handle)));
}

/// Update emitters for PlanarProjection (Decal) primitive type
pub fn update_emitter_decal(
    mut commands: Commands,
    res_assets_vfx_system_definition_data: Res<Assets<VfxSystemDefinitionData>>,
    res_asset_server: Res<AssetServer>,
    mut res_resource_cache: ResMut<ResourceCache>,
    mut res_unlit_decal_material: ResMut<Assets<ParticleMaterialUnlitDecal>>,
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
        if emitter_type != EmitterType::Decal {
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
            .map(|v| res_resource_cache.get_image_srgb(&res_asset_server, v));

        let particle_color_texture = vfx_emitter_definition_data
            .particle_color_texture
            .as_ref()
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

            // Extract m_projection from PlanarProjection
            if let EnumVfxPrimitive::VfxPrimitivePlanarProjection(VfxPrimitivePlanarProjection {
                ref m_projection,
            }) = primitive
            {
                if let Some(m_projection) = m_projection {
                    attach_unlit_decal_visuals(
                        &mut commands,
                        particle_entity,
                        m_projection,
                        texture.clone(),
                        particle_color_texture.clone(),
                        blend_mode,
                        &mut res_unlit_decal_material,
                    );
                }
            }
        }
    }
}
