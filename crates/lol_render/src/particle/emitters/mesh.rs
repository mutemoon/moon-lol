use bevy::prelude::*;
use league_core::extract::{
    EnumVfxPrimitive, VfxEmitterDefinitionData, VfxPrimitiveMesh, VfxSystemDefinitionData,
};
use lol_core::lifetime::Lifetime;

use super::state::ParticleEmitterState;
use super::utils::{
    calculate_emission_params, calculate_particle_transform_frame, get_emitter_type,
    spawn_particle_entity, EmissionParams, EmitterType, ParticleBirthParams,
};
use crate::particle::particle::mesh::{
    ParticleMaterialMesh, UniformsPixelMesh, UniformsVertexMesh,
};
use crate::particle::utils::create_black_pixel_texture;
use crate::particle::ParticleId;
use crate::resource::ResourceCache;

pub fn attach_mesh_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    _vfx_emitter_definition_data: &VfxEmitterDefinitionData,
    m_mesh: &VfxPrimitiveMesh,
    texture: Option<Handle<Image>>,
    particle_color_texture: Option<Handle<Image>>,
    blend_mode: u8,
    _res_mesh: &mut ResMut<Assets<Mesh>>,
    res_image: &mut ResMut<Assets<Image>>,
    res_particle_material_mesh: &mut ResMut<Assets<ParticleMaterialMesh>>,
    res_resource_cache: &mut ResMut<ResourceCache>,
    res_asset_server: &Res<AssetServer>,
) {
    let Some(m_mesh_data) = &m_mesh.m_mesh else {
        println!("VfxPrimitiveMesh: m_mesh is None");
        return;
    };

    let Some(mesh_name) = &m_mesh_data.m_simple_mesh_name else {
        println!("VfxPrimitiveMesh: m_simple_mesh_name is None");
        return;
    };

    let mesh = res_resource_cache.get_mesh(res_asset_server, mesh_name);
    let black_pixel_texture = res_image.add(create_black_pixel_texture());

    commands.entity(particle_entity).insert((
        Mesh3d(mesh),
        MeshMaterial3d(res_particle_material_mesh.add(ParticleMaterialMesh {
            uniforms_vertex: UniformsVertexMesh::default(),
            uniforms_pixel: UniformsPixelMesh::default(),
            texture: texture.clone(),
            particle_color_texture: particle_color_texture.clone(),
            cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: Some(black_pixel_texture),
            cmb_tex_fow_map_smp_clamp_no_mip: None,
            blend_mode,
        })),
    ));
}

/// Update emitters for Mesh primitive type
pub fn update_emitter_mesh(
    mut commands: Commands,
    mut res_mesh: ResMut<Assets<Mesh>>,
    res_assets_vfx_system_definition_data: Res<Assets<VfxSystemDefinitionData>>,
    res_asset_server: Res<AssetServer>,
    mut res_resource_cache: ResMut<ResourceCache>,
    mut res_image: ResMut<Assets<Image>>,
    mut res_particle_material_mesh: ResMut<Assets<ParticleMaterialMesh>>,
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
        if emitter_type != EmitterType::Mesh {
            continue;
        }

        let primitive = vfx_emitter_definition_data
            .primitive
            .clone()
            .unwrap_or(EnumVfxPrimitive::VfxPrimitiveCameraUnitQuad);

        // Extract m_mesh from Mesh primitive
        let m_mesh = match primitive {
            EnumVfxPrimitive::VfxPrimitiveMesh(ref m) => m,
            _ => continue,
        };

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

            // Extract m_mesh from Mesh primitive
            attach_mesh_visuals(
                &mut commands,
                particle_entity,
                vfx_emitter_definition_data,
                m_mesh,
                texture.clone(),
                particle_color_texture.clone(),
                blend_mode,
                &mut res_mesh,
                &mut res_image,
                &mut res_particle_material_mesh,
                &mut res_resource_cache,
                &res_asset_server,
            );
        }
    }
}
