use bevy::prelude::*;
use league_core::extract::{
    EnumVfxPrimitive, EnumVfxShape, Unk0xee39916f, VfxEmitterDefinitionData,
    VfxPrimitivePlanarProjection, VfxShapeBox, VfxShapeCylinder, VfxShapeLegacy,
};

use super::state::ParticleEmitterState;
use crate::core::lifetime::Lifetime;
use crate::core::particle::particle::ParticleState;
use crate::core::particle::utils::StochasticSampler;
use crate::core::particle::ParticleId;

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
pub fn get_emitter_type(vfx_emitter_definition_data: &VfxEmitterDefinitionData) -> EmitterType {
    let primitive = vfx_emitter_definition_data
        .primitive
        .clone()
        .unwrap_or(EnumVfxPrimitive::VfxPrimitiveCameraUnitQuad);

    match primitive {
        // Quad primitives - check if it's a distortion effect
        EnumVfxPrimitive::VfxPrimitiveArbitraryQuad
        | EnumVfxPrimitive::VfxPrimitiveCameraUnitQuad => {
            if vfx_emitter_definition_data.distortion_definition.is_some() {
                EmitterType::Distortion
            } else {
                EmitterType::Quad
            }
        }
        // Mesh primitives
        EnumVfxPrimitive::VfxPrimitiveMesh(_) => EmitterType::Mesh,
        EnumVfxPrimitive::VfxPrimitiveAttachedMesh(_) => EmitterType::SkinnedMesh,
        // Decal primitives
        EnumVfxPrimitive::VfxPrimitivePlanarProjection { .. } => EmitterType::Decal,
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
    vfx_emitter_definition_data: &VfxEmitterDefinitionData,
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
    vfx_emitter_definition_data: &VfxEmitterDefinitionData,
    primitive: &EnumVfxPrimitive,
    progress: f32,
) -> (Transform, Vec3, f32) {
    let mut birth_scale0 = if is_uniform_scale {
        Vec3::splat(birth_params.birth_scale0.x)
    } else {
        birth_params.birth_scale0
    };

    let translation = vfx_emitter_definition_data
        .spawn_shape
        .clone()
        .and_then(|v| match v {
            EnumVfxShape::Unk0xee39916f(Unk0xee39916f { emit_offset }) => emit_offset,
            EnumVfxShape::VfxShapeLegacy(VfxShapeLegacy { emit_offset, .. }) => emit_offset
                .and_then(|v| Some(StochasticSampler::<Vec3>::from(v).sample_clamped(progress))),
            EnumVfxShape::VfxShapeBox(VfxShapeBox { .. }) => Some(Vec3::ZERO),
            EnumVfxShape::VfxShapeCylinder(VfxShapeCylinder { .. }) => Some(Vec3::ZERO),
            _ => todo!(),
        })
        .unwrap_or(Vec3::ZERO);

    let rotation_quat = Quat::from_euler(
        EulerRot::XYZEx,
        birth_params.birth_rotation0.x.to_radians(),
        (birth_params.birth_rotation0.y - birth_params.birth_rotation0.z).to_radians(),
        0.,
    );

    if let EnumVfxPrimitive::VfxPrimitivePlanarProjection(VfxPrimitivePlanarProjection {
        ref m_projection,
    }) = primitive
    {
        birth_scale0.x = birth_scale0.x * 2.;
        birth_scale0.y = m_projection.as_ref().unwrap().m_y_range.unwrap();
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
