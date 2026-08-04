use bevy::prelude::*;
use lol_base_render::particle::{ConfigVfxEmitterDefinition, ConfigVfxPrimitive, ConfigVfxShape};
use lol_core::lifetime::Lifetime;

use super::state::ParticleEmitterState;

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

/// 计算发射位置和发射角度 (translation, shape_rotation)
pub fn calculate_spawn_pose(spawn_shape: Option<&ConfigVfxShape>, progress: f32) -> (Vec3, Quat) {
    let mut shape_rotation = Quat::IDENTITY;
    let mut raw_translation = Vec3::ZERO;

    if let Some(shape) = spawn_shape {
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
                for (angle_sampler, axis) in
                    emit_rotation_angles.iter().zip(emit_rotation_axes.iter())
                {
                    let deg = angle_sampler.sample_clamped(progress);
                    if deg != 0.0 && *axis != Vec3::ZERO {
                        let normalized_axis = axis.normalize();
                        let q = Quat::from_axis_angle(normalized_axis, deg.to_radians());
                        shape_rotation = shape_rotation * q;
                    }
                }
            }
            ConfigVfxShape::Box { size, .. } => {
                let size = size.unwrap_or(Vec3::ZERO);
                raw_translation = Vec3::new(
                    (rand::random::<f32>() - 0.5) * size.x,
                    (rand::random::<f32>() - 0.5) * size.y,
                    (rand::random::<f32>() - 0.5) * size.z,
                );
            }
            ConfigVfxShape::Sphere { radius, .. } => {
                let radius = radius.unwrap_or(0.0);
                if radius > 0.0 {
                    let u: f32 = rand::random();
                    let v: f32 = rand::random();
                    let theta = u * std::f32::consts::TAU;
                    let phi = (2.0 * v - 1.0_f32).acos();
                    let r = radius * rand::random::<f32>().cbrt();
                    let sin_phi = phi.sin();
                    raw_translation = Vec3::new(
                        r * sin_phi * theta.cos(),
                        r * sin_phi * theta.sin(),
                        r * phi.cos(),
                    );
                }
            }
            ConfigVfxShape::Cylinder { height, radius, .. } => {
                let radius = radius.unwrap_or(0.0);
                let height = height.unwrap_or(0.0);
                let r_sample = radius * rand::random::<f32>().sqrt();
                let theta = rand::random::<f32>() * std::f32::consts::TAU;
                let y = (rand::random::<f32>() - 0.5) * height;
                raw_translation = Vec3::new(r_sample * theta.cos(), y, r_sample * theta.sin());
            }
        }
    }

    let translation = shape_rotation * raw_translation;
    (translation, shape_rotation)
}

/// 计算粒子的初始 Transform
pub fn calculate_particle_transform(
    birth_rotation0: Vec3,
    mut birth_scale0: Vec3,
    translation: Vec3,
    primitive: &ConfigVfxPrimitive,
) -> Transform {
    let rotation_quat = Quat::from_euler(
        EulerRot::XYZEx,
        birth_rotation0.x.to_radians(),
        birth_rotation0.y.to_radians(),
        birth_rotation0.z.to_radians(),
    );

    if let ConfigVfxPrimitive::VfxPrimitivePlanarProjection { y_range } = primitive {
        birth_scale0.x *= 2.0;
        birth_scale0.y = y_range.unwrap_or(1.0);
        birth_scale0.z *= 2.0;
    }

    Transform::from_rotation(rotation_quat)
        .with_translation(translation)
        .with_scale(birth_scale0)
}

pub fn calculate_particle_transform_frame(
    raw_rotation0: Vec3,
    raw_scale0: Vec3,
    is_uniform_scale: bool,
    vfx_emitter_definition_data: &ConfigVfxEmitterDefinition,
    primitive: &ConfigVfxPrimitive,
    progress: f32,
) -> (Transform, Quat, Vec3, f32) {
    let mut birth_scale0 = if is_uniform_scale {
        Vec3::splat(raw_scale0.x)
    } else {
        raw_scale0
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

    // 1. 计算发射位置和发射角度
    let (translation, shape_rotation) =
        calculate_spawn_pose(vfx_emitter_definition_data.spawn_shape.as_ref(), progress);

    // 2. 计算粒子的初始 transform
    let transform =
        calculate_particle_transform(raw_rotation0, birth_scale0, translation, primitive);

    let num_frames = vfx_emitter_definition_data.num_frames.unwrap_or(0) as f32;
    let frame = if vfx_emitter_definition_data
        .is_random_start_frame
        .unwrap_or(false)
    {
        (num_frames * rand::random::<f32>()).floor()
    } else {
        (num_frames * progress).floor()
    };

    (transform, shape_rotation, transform.scale, frame)
}
