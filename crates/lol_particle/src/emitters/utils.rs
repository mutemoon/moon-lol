use bevy::prelude::*;
use lol_base_render::particle::{ConfigVfxEmitterDefinition, ConfigVfxPrimitive, ConfigVfxShape};
use lol_core::lifetime::Lifetime;

use super::state::ParticleEmitterState;
use crate::ParticleId;
use crate::particle::ParticleState;

/// Emitter type classification for particle systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitterType {
    Quad,
    Mesh,
    SkinnedMesh,
    Decal,
    Distortion,
    Unknown,
}

/// Extract the emitter type from VFX emitter definition data
pub fn get_emitter_type(vfx_emitter_definition_data: &ConfigVfxEmitterDefinition) -> EmitterType {
    let primitive = vfx_emitter_definition_data
        .primitive
        .clone()
        .unwrap_or(ConfigVfxPrimitive::VfxPrimitiveCameraUnitQuad);

    match primitive {
        // Quad primitives - check if it's a distortion effect
        ConfigVfxPrimitive::VfxPrimitiveArbitraryQuad
        | ConfigVfxPrimitive::VfxPrimitiveCameraUnitQuad => {
            if vfx_emitter_definition_data.distortion_definition.is_some() {
                EmitterType::Distortion
            } else {
                EmitterType::Quad
            }
        }
        // Mesh primitives
        ConfigVfxPrimitive::VfxPrimitiveMesh { .. } => EmitterType::Mesh,
        ConfigVfxPrimitive::VfxPrimitiveAttachedMesh { .. } => EmitterType::SkinnedMesh,
        // Decal primitives
        ConfigVfxPrimitive::VfxPrimitivePlanarProjection { .. } => EmitterType::Decal,
        // Unknown/unsupported types
        _ => EmitterType::Unknown,
    }
}

pub struct ParticleBirthParams {
    pub birth_color: Vec4,
    pub birth_velocity: Vec3,
    pub birth_acceleration: Vec3,
    pub birth_rotation0: Vec3,
    pub birth_scale0: Vec3,
    pub birth_uv_offset: Vec2,
    pub birth_uv_scroll_rate: Vec2,
}

impl ParticleBirthParams {
    pub fn sample(emitter: &mut ParticleEmitterState, progress: f32) -> Self {
        Self {
            birth_color: emitter.birth_color.sample_clamped(progress),
            birth_velocity: emitter.birth_velocity.sample_clamped(progress),
            birth_acceleration: emitter.birth_acceleration.sample_clamped(progress),
            birth_rotation0: emitter.birth_rotation0.sample_clamped(progress),
            birth_scale0: emitter.birth_scale0.sample_clamped(progress),
            birth_uv_offset: emitter.birth_uv_offset.sample_clamped(progress),
            birth_uv_scroll_rate: emitter.birth_uv_scroll_rate.sample_clamped(progress),
        }
    }
}

pub fn spawn_particle_entity(
    commands: &mut Commands,
    particle_id: &ParticleId,
    emitter_entity: Entity,
    particle_lifetime: f32,
    transform: Transform,
    frame: f32,
    birth_params: &ParticleBirthParams,
    adjusted_birth_scale0: Vec3,
) -> Entity {
    let particle_state = ParticleState {
        birth_uv_offset: birth_params.birth_uv_offset,
        birth_uv_scroll_rate: birth_params.birth_uv_scroll_rate,
        birth_color: birth_params.birth_color,
        birth_scale0: adjusted_birth_scale0,
        initial_rotation: transform.rotation,
        velocity: birth_params.birth_velocity,
        acceleration: birth_params.birth_acceleration,
        frame,
    };

    commands
        .spawn((
            particle_id.clone(),
            particle_state,
            Lifetime::new_timer(particle_lifetime),
            transform,
            Pickable::IGNORE,
            ChildOf(emitter_entity),
        ))
        .id()
}

/// Emission parameters calculated for spawning particles
pub struct EmissionParams {
    pub particles_to_spawn: u32,
    pub progress: f32,
}

/// Calculate emission parameters for an emitter
pub fn calculate_emission_params(
    lifetime: &Lifetime,
    emitter: &mut ParticleEmitterState,
    vfx_emitter_definition_data: &ConfigVfxEmitterDefinition,
    delta_secs: f32,
) -> Option<EmissionParams> {
    if lifetime.is_dead() {
        return None;
    }

    let progress = lifetime.progress();
    let rate = emitter.rate.sample_clamped(progress);

    let is_single_particle = vfx_emitter_definition_data
        .is_single_particle
        .unwrap_or(false);

    let particles_to_spawn_f32 = rate * delta_secs + emitter.emission_debt;

    let particles_to_spawn = if is_single_particle {
        // Note: caller is responsible for marking lifetime as dead
        rate as u32
    } else {
        particles_to_spawn_f32.floor() as u32
    };

    emitter.emission_debt = particles_to_spawn_f32.fract();

    if particles_to_spawn == 0 {
        return None;
    }

    Some(EmissionParams {
        particles_to_spawn,
        progress,
    })
}

pub fn calculate_particle_transform_frame(
    birth_params: &ParticleBirthParams,
    is_uniform_scale: bool,
    vfx_emitter_definition_data: &ConfigVfxEmitterDefinition,
    primitive: &ConfigVfxPrimitive,
    progress: f32,
) -> (Transform, Vec3, f32) {
    let mut birth_scale0 = if is_uniform_scale {
        Vec3::splat(birth_params.birth_scale0.x)
    } else {
        birth_params.birth_scale0
    };

    if let Some(flex) = &vfx_emitter_definition_data.flex_shape_definition {
        if let Some(scale_size) = flex.scale_birth_scale_by_bound_object_size {
            birth_scale0 *= 1.0 + scale_size;
        }
        if let Some(scale_height) = flex.scale_birth_scale_by_bound_object_height {
            birth_scale0.y *= 1.0 + scale_height;
        }
        if let Some(scale_radius) = flex.scale_birth_scale_by_bound_object_radius {
            birth_scale0.x *= 1.0 + scale_radius;
            birth_scale0.z *= 1.0 + scale_radius;
        }
    }

    let mut shape_rotation = Quat::IDENTITY;
    let mut raw_translation = Vec3::ZERO;

    if let Some(shape) = &vfx_emitter_definition_data.spawn_shape {
        match shape {
            ConfigVfxShape::Unk0xee39916f { emit_offset } => {
                raw_translation = emit_offset.unwrap_or(Vec3::ZERO);
            }
            ConfigVfxShape::Legacy {
                emit_offset,
                emit_rotation_angles,
                emit_rotation_axes,
            } => {
                raw_translation = emit_offset.sample_clamped(progress);
                // for (angle_sampler, axis) in
                //     emit_rotation_angles.iter().zip(emit_rotation_axes.iter())
                // {
                //     let deg = angle_sampler.sample_clamped(progress);
                //     if deg != 0.0 && *axis != Vec3::ZERO {
                //         let normalized_axis = axis.normalize();
                //         let q = Quat::from_axis_angle(normalized_axis, deg.to_radians());
                //         shape_rotation = shape_rotation * q;
                //     }
                // }
            }
            ConfigVfxShape::Box { size, .. } => {
                raw_translation = size.unwrap_or(Vec3::ZERO);
            }
            _ => {}
        }
    }

    let translation = shape_rotation * raw_translation;

    let base_rotation_quat = Quat::from_euler(
        EulerRot::XYZEx,
        birth_params.birth_rotation0.x.to_radians(),
        birth_params.birth_rotation0.y.to_radians(),
        birth_params.birth_rotation0.z.to_radians(),
    );
    let rotation_quat = shape_rotation * base_rotation_quat;

    if let ConfigVfxPrimitive::VfxPrimitivePlanarProjection { y_range } = primitive {
        birth_scale0.x = birth_scale0.x * 2.;
        birth_scale0.y = y_range.unwrap_or(1.0);
        birth_scale0.z = birth_scale0.z * 2.;
    }

    let transform = Transform::from_rotation(rotation_quat)
        .with_translation(translation)
        .with_scale(birth_scale0);

    let num_frames = vfx_emitter_definition_data.num_frames.unwrap_or(0) as f32;
    let frame = if vfx_emitter_definition_data
        .is_random_start_frame
        .unwrap_or(false)
    {
        (num_frames * rand::random::<f32>()).floor()
    } else {
        (num_frames * progress).floor()
    };

    (transform, birth_scale0, frame)
}
