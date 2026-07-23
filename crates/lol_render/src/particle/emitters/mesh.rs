use bevy::prelude::*;
use lol_base::particle::{
    ConfigVfxEmitterDefinition, ConfigVfxPrimitive, ConfigVfxSystemDefinition,
};
use lol_core::lifetime::Lifetime;

use super::state::ParticleEmitterState;
use super::utils::{
    EmissionParams, EmitterType, ParticleBirthParams, calculate_emission_params,
    calculate_particle_transform_frame, get_emitter_type, spawn_particle_entity,
};
use crate::particle::ParticleId;
use crate::particle::particle::mesh::{
    ParticleMaterialMesh, UniformsPixelMesh, UniformsVertexMesh,
};
use crate::particle::utils::{ResourceCache, create_black_pixel_texture};

pub fn attach_mesh_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    _vfx_emitter_definition_data: &ConfigVfxEmitterDefinition,
    mesh_name: Option<&str>,
    texture: Option<Handle<Image>>,
    particle_color_texture: Option<Handle<Image>>,
    blend_mode: u8,
    _res_mesh: &mut ResMut<Assets<Mesh>>,
    res_image: &mut ResMut<Assets<Image>>,
    res_particle_material_mesh: &mut ResMut<Assets<ParticleMaterialMesh>>,
    res_resource_cache: &mut ResMut<ResourceCache>,
    res_asset_server: &Res<AssetServer>,
) {
    let Some(mesh_name) = mesh_name else {
        println!("VfxPrimitiveMesh: mesh_name is None");
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
    res_assets_vfx_system_definition_data: Res<Assets<ConfigVfxSystemDefinition>>,
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
            .unwrap_or(ConfigVfxPrimitive::VfxPrimitiveCameraUnitQuad);

        // Extract simple_mesh_name from Mesh primitive
        let simple_mesh_name = match &primitive {
            ConfigVfxPrimitive::VfxPrimitiveMesh {
                simple_mesh_name, ..
            } => simple_mesh_name.as_deref(),
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

            attach_mesh_visuals(
                &mut commands,
                particle_entity,
                vfx_emitter_definition_data,
                simple_mesh_name,
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
