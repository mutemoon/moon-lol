use bevy::prelude::*;
use bevy::animation::AnimationTarget;
use bevy::mesh::skinning::SkinnedMesh;
use league_core::{EnumVfxPrimitive, VfxEmitterDefinitionData, VfxSystemDefinitionData};

use crate::{
    create_black_pixel_texture, spawn_shadow_skin_entity, AssetServerLoadLeague, Lifetime,
    ParticleId, ParticleMaterialSkinnedMeshParticle, ResourceCache,
    UniformsPixelSkinnedMeshParticle, UniformsVertexSkinnedMeshParticle,
};

use super::{EmitterOf, ParticleEmitterState};
use super::utils::{ParticleBirthParams, EmissionParams, calculate_emission_params, calculate_particle_transform_frame, spawn_particle_entity, get_emitter_type, EmitterType};

pub fn attach_skinned_mesh_visuals(
    commands: &mut Commands,
    particle_entity: Entity,
    emitter_of: &EmitterOf,
    vfx_emitter_definition_data: &VfxEmitterDefinitionData,
    texture: Option<Handle<Image>>,
    particle_color_texture: Option<Handle<Image>>,
    blend_mode: u8,
    res_image: &mut ResMut<Assets<Image>>,
    res_particle_material_skinned_mesh_particle: &mut ResMut<Assets<ParticleMaterialSkinnedMeshParticle>>,
    res_asset_server: &Res<AssetServer>,
    q_mesh3d: Query<&Mesh3d>,
    q_skinned_mesh: Query<&SkinnedMesh>,
    q_children: Query<&Children>,
    q_animation_target: Query<(Entity, &Transform, &AnimationTarget)>,
) {
    let black_pixel_texture = res_image.add(create_black_pixel_texture());

    // Handle material overrides
    let final_texture = if let Some(material_override_definitions) =
        &vfx_emitter_definition_data.material_override_definitions
    {
        let mut tex = texture;
        for material_override_definition in material_override_definitions {
            if let Some(base_texture) = &material_override_definition.base_texture {
                tex = Some(res_asset_server.load_league(base_texture));
            }
        }
        tex
    } else {
        texture
    };

    let material = MeshMaterial3d(res_particle_material_skinned_mesh_particle.add(
        ParticleMaterialSkinnedMeshParticle {
            uniforms_vertex: UniformsVertexSkinnedMeshParticle::default(),
            uniforms_pixel: UniformsPixelSkinnedMeshParticle::default(),
            texture: final_texture,
            particle_color_texture: particle_color_texture.clone(),
            cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: Some(black_pixel_texture),
            cmb_tex_fow_map_smp_clamp_no_mip: None,
            blend_mode,
        },
    ));

    spawn_shadow_skin_entity(
        commands,
        particle_entity,
        emitter_of.0,
        material,
        q_mesh3d,
        q_skinned_mesh,
        q_children,
        q_animation_target,
    );
}

/// Update emitters for AttachedMesh (SkinnedMesh) primitive type
pub fn update_emitter_skinned_mesh(
    mut commands: Commands,
    res_assets_vfx_system_definition_data: Res<Assets<VfxSystemDefinitionData>>,
    res_asset_server: Res<AssetServer>,
    mut res_resource_cache: ResMut<ResourceCache>,
    mut res_image: ResMut<Assets<Image>>,
    mut res_particle_material_skinned_mesh_particle: ResMut<
        Assets<ParticleMaterialSkinnedMeshParticle>,
    >,
    mut query: Query<(
        Entity,
        &EmitterOf,
        &mut Lifetime,
        &mut ParticleEmitterState,
        &ParticleId,
    )>,
    q_mesh3d: Query<&Mesh3d>,
    q_skinned_mesh: Query<&SkinnedMesh>,
    q_children: Query<&Children>,
    q_animation_target: Query<(Entity, &Transform, &AnimationTarget)>,
    time: Res<Time>,
) {
    for (emitter_entity, emitter_of, mut lifetime, mut emitter, particle_id) in query.iter_mut() {
        let vfx_emitter_definition_data =
            particle_id.get_def(&res_assets_vfx_system_definition_data);

        // Check if this emitter should be processed by this update function
        let emitter_type = get_emitter_type(vfx_emitter_definition_data);
        if emitter_type != EmitterType::SkinnedMesh {
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
            .map(|v| res_resource_cache.get_image(&res_asset_server, v));

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

            attach_skinned_mesh_visuals(
                &mut commands,
                particle_entity,
                emitter_of,
                vfx_emitter_definition_data,
                texture.clone(),
                particle_color_texture.clone(),
                blend_mode,
                &mut res_image,
                &mut res_particle_material_skinned_mesh_particle,
                &res_asset_server,
                q_mesh3d,
                q_skinned_mesh,
                q_children,
                q_animation_target,
            );
        }
    }
}
