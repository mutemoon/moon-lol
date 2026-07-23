use bevy::prelude::*;
use league_core::extract::{
    EnumVfxPrimitive, EnumVfxShape, Unk0xee39916f, ValueColor, ValueFloat, ValueVector2,
    ValueVector3, VfxDistortionDefinitionData, VfxEmitterDefinitionData,
    VfxMaterialOverrideDefinitionData, VfxPrimitiveAttachedMesh, VfxPrimitiveMesh,
    VfxPrimitivePlanarProjection, VfxProbabilityTableData, VfxShapeBox, VfxShapeCylinder,
    VfxShapeLegacy, VfxShapeSphere, VfxSystemDefinitionData, VfxTextureMultDefinitionData,
};
use lol_base::particle::{
    ConfigVfxDistortionDefinition, ConfigVfxEmitterDefinition, ConfigVfxMaterialOverride,
    ConfigVfxPrimitive, ConfigVfxShape, ConfigVfxSystemDefinition, ConfigVfxTextureMult,
    ProbabilityCurve, Sampler, StochasticSampler,
};

fn convert_sampler_float(value: &ValueFloat, default: f32) -> Sampler<f32> {
    let constant_val = value.constant_value.unwrap_or(default);
    if let Some(ref dynamics) = value.dynamics {
        match dynamics.times.len() {
            0 => Sampler::Constant(constant_val),
            1 => Sampler::Constant(dynamics.values.first().cloned().unwrap_or(constant_val)),
            _ => {
                let samples: Vec<(f32, f32)> = dynamics
                    .times
                    .iter()
                    .cloned()
                    .zip(dynamics.values.iter().cloned())
                    .collect();
                Sampler::new_curve(samples).unwrap_or(Sampler::Constant(constant_val))
            }
        }
    } else {
        Sampler::Constant(constant_val)
    }
}

fn convert_sampler_vector3(value: &ValueVector3, default: Vec3) -> Sampler<Vec3> {
    let constant_val = value.constant_value.unwrap_or(default);
    if let Some(ref dynamics) = value.dynamics {
        match dynamics.times.len() {
            0 => Sampler::Constant(constant_val),
            1 => Sampler::Constant(dynamics.values.first().cloned().unwrap_or(constant_val)),
            _ => {
                let samples: Vec<(f32, Vec3)> = dynamics
                    .times
                    .iter()
                    .cloned()
                    .zip(dynamics.values.iter().cloned())
                    .collect();
                Sampler::new_curve(samples).unwrap_or(Sampler::Constant(constant_val))
            }
        }
    } else {
        Sampler::Constant(constant_val)
    }
}

fn convert_sampler_vector2(value: &ValueVector2, default: Vec2) -> Sampler<Vec2> {
    let constant_val = value.constant_value.unwrap_or(default);
    if let Some(ref dynamics) = value.dynamics {
        match dynamics.times.len() {
            0 => Sampler::Constant(constant_val),
            1 => Sampler::Constant(dynamics.values.first().cloned().unwrap_or(constant_val)),
            _ => {
                let samples: Vec<(f32, Vec2)> = dynamics
                    .times
                    .iter()
                    .cloned()
                    .zip(dynamics.values.iter().cloned())
                    .collect();
                Sampler::new_curve(samples).unwrap_or(Sampler::Constant(constant_val))
            }
        }
    } else {
        Sampler::Constant(constant_val)
    }
}

fn convert_sampler_color(value: &ValueColor, default: Vec4) -> Sampler<Vec4> {
    let constant_val = value.constant_value.unwrap_or(default);
    if let Some(ref dynamics) = value.dynamics {
        if let (Some(times), Some(values)) = (&dynamics.times, &dynamics.values) {
            match times.len() {
                0 => Sampler::Constant(constant_val),
                1 => Sampler::Constant(values.first().cloned().unwrap_or(constant_val)),
                _ => {
                    let samples: Vec<(f32, Vec4)> =
                        times.iter().cloned().zip(values.iter().cloned()).collect();
                    Sampler::new_curve(samples).unwrap_or(Sampler::Constant(constant_val))
                }
            }
        } else {
            Sampler::Constant(constant_val)
        }
    } else {
        Sampler::Constant(constant_val)
    }
}

fn convert_probability_table(table: &VfxProbabilityTableData) -> Option<ProbabilityCurve> {
    let (times, values) = match (&table.key_times, &table.key_values) {
        (Some(times), Some(values)) => (times.clone(), values.clone()),
        _ => return None,
    };
    if times.len() != values.len() {
        return None;
    }
    match times.len() {
        0 => None,
        1 => Some(ProbabilityCurve::Constant(values[0])),
        _ => {
            let samples: Vec<(f32, f32)> = times.into_iter().zip(values).collect();
            ProbabilityCurve::new_curve(samples).ok()
        }
    }
}

pub fn convert_stochastic_float(
    value: &Option<ValueFloat>,
    default: f32,
) -> StochasticSampler<f32> {
    let val = value.clone().unwrap_or(ValueFloat {
        dynamics: None,
        constant_value: Some(default),
    });
    let base_sampler = convert_sampler_float(&val, default);
    let mut prob_curves = Vec::new();
    if let Some(ref dynamics) = val.dynamics {
        if let Some(ref tables) = dynamics.probability_tables {
            for table in tables {
                prob_curves.push(convert_probability_table(table));
            }
        }
    }
    StochasticSampler {
        base_sampler,
        prob_curves,
    }
}

pub fn convert_stochastic_vector3(
    value: &Option<ValueVector3>,
    default: Vec3,
) -> StochasticSampler<Vec3> {
    let val = value.clone().unwrap_or(ValueVector3 {
        dynamics: None,
        constant_value: Some(default),
    });
    let base_sampler = convert_sampler_vector3(&val, default);
    let mut prob_curves = Vec::new();
    if let Some(ref dynamics) = val.dynamics {
        if let Some(ref tables) = dynamics.probability_tables {
            for table in tables {
                prob_curves.push(convert_probability_table(table));
            }
        }
    }
    StochasticSampler {
        base_sampler,
        prob_curves,
    }
}

pub fn convert_stochastic_vector2(
    value: &Option<ValueVector2>,
    default: Vec2,
) -> StochasticSampler<Vec2> {
    let val = value.clone().unwrap_or(ValueVector2 {
        dynamics: None,
        constant_value: Some(default),
    });
    let base_sampler = convert_sampler_vector2(&val, default);
    let mut prob_curves = Vec::new();
    if let Some(ref dynamics) = val.dynamics {
        if let Some(ref tables) = dynamics.probability_tables {
            for table in tables {
                prob_curves.push(convert_probability_table(table));
            }
        }
    }
    StochasticSampler {
        base_sampler,
        prob_curves,
    }
}

pub fn convert_stochastic_color(
    value: &Option<ValueColor>,
    default: Vec4,
) -> StochasticSampler<Vec4> {
    let val = value.clone().unwrap_or(ValueColor {
        dynamics: None,
        constant_value: Some(default),
    });
    let base_sampler = convert_sampler_color(&val, default);
    let mut prob_curves = Vec::new();
    if let Some(ref dynamics) = val.dynamics {
        if let Some(ref tables) = dynamics.probability_tables {
            for table in tables {
                prob_curves.push(convert_probability_table(table));
            }
        }
    }
    StochasticSampler {
        base_sampler,
        prob_curves,
    }
}

fn convert_shape(shape: &Option<EnumVfxShape>) -> Option<ConfigVfxShape> {
    shape.as_ref().map(|s| match s {
        EnumVfxShape::VfxShapeBox(VfxShapeBox { flags, size }) => ConfigVfxShape::Box {
            flags: *flags,
            size: *size,
        },
        EnumVfxShape::VfxShapeCylinder(VfxShapeCylinder {
            flags,
            height,
            radius,
        }) => ConfigVfxShape::Cylinder {
            flags: *flags,
            height: *height,
            radius: *radius,
        },
        EnumVfxShape::VfxShapeLegacy(VfxShapeLegacy {
            emit_offset,
            emit_rotation_angles,
            emit_rotation_axes,
        }) => {
            let offset = convert_stochastic_vector3(emit_offset, Vec3::ZERO);
            let mut angles = Vec::new();
            if let Some(list) = emit_rotation_angles {
                for angle in list {
                    angles.push(convert_stochastic_float(&Some(angle.clone()), 0.0));
                }
            }
            let axes = emit_rotation_axes.clone().unwrap_or_default();
            ConfigVfxShape::Legacy {
                emit_offset: offset,
                emit_rotation_angles: angles,
                emit_rotation_axes: axes,
            }
        }
        EnumVfxShape::VfxShapeSphere(VfxShapeSphere { flags, radius }) => ConfigVfxShape::Sphere {
            flags: *flags,
            radius: *radius,
        },
        EnumVfxShape::Unk0xee39916f(Unk0xee39916f { emit_offset }) => {
            ConfigVfxShape::Unk0xee39916f {
                emit_offset: *emit_offset,
            }
        }
        _ => ConfigVfxShape::Unk0xee39916f { emit_offset: None },
    })
}

fn convert_primitive(primitive: &Option<EnumVfxPrimitive>) -> Option<ConfigVfxPrimitive> {
    primitive.as_ref().map(|p| match p {
        EnumVfxPrimitive::Unk0x8df5fcf7 => ConfigVfxPrimitive::Unk0x8df5fcf7,
        EnumVfxPrimitive::VfxPrimitiveArbitraryQuad => {
            ConfigVfxPrimitive::VfxPrimitiveArbitraryQuad
        }
        EnumVfxPrimitive::VfxPrimitiveArbitraryTrail(_) => {
            ConfigVfxPrimitive::VfxPrimitiveArbitraryTrail
        }
        EnumVfxPrimitive::VfxPrimitiveAttachedMesh(VfxPrimitiveAttachedMesh {
            align_pitch_to_camera,
            align_yaw_to_camera,
            m_mesh,
            ..
        }) => ConfigVfxPrimitive::VfxPrimitiveAttachedMesh {
            align_pitch_to_camera: *align_pitch_to_camera,
            align_yaw_to_camera: *align_yaw_to_camera,
            simple_mesh_name: m_mesh.as_ref().and_then(|m| m.m_simple_mesh_name.clone()),
        },
        EnumVfxPrimitive::VfxPrimitiveBeam(_) => ConfigVfxPrimitive::VfxPrimitiveBeam,
        EnumVfxPrimitive::VfxPrimitiveCameraSegmentBeam(_) => {
            ConfigVfxPrimitive::VfxPrimitiveCameraSegmentBeam
        }
        EnumVfxPrimitive::VfxPrimitiveCameraTrail(_) => ConfigVfxPrimitive::VfxPrimitiveCameraTrail,
        EnumVfxPrimitive::VfxPrimitiveCameraUnitQuad => {
            ConfigVfxPrimitive::VfxPrimitiveCameraUnitQuad
        }
        EnumVfxPrimitive::VfxPrimitiveMesh(VfxPrimitiveMesh {
            align_pitch_to_camera,
            align_yaw_to_camera,
            m_mesh,
            ..
        }) => ConfigVfxPrimitive::VfxPrimitiveMesh {
            align_pitch_to_camera: *align_pitch_to_camera,
            align_yaw_to_camera: *align_yaw_to_camera,
            simple_mesh_name: m_mesh.as_ref().and_then(|m| m.m_simple_mesh_name.clone()),
        },
        EnumVfxPrimitive::VfxPrimitivePlanarProjection(VfxPrimitivePlanarProjection {
            m_projection,
        }) => ConfigVfxPrimitive::VfxPrimitivePlanarProjection {
            y_range: m_projection.as_ref().and_then(|proj| proj.m_y_range),
        },
        EnumVfxPrimitive::VfxPrimitiveRay => ConfigVfxPrimitive::VfxPrimitiveRay,
    })
}

fn convert_distortion(
    dist: &Option<VfxDistortionDefinitionData>,
) -> Option<ConfigVfxDistortionDefinition> {
    dist.as_ref().map(|d| ConfigVfxDistortionDefinition {
        distortion: d.distortion,
        distortion_mode: d.distortion_mode,
        normal_map_texture: d.normal_map_texture.clone(),
    })
}

fn convert_texture_mult(
    mult: &Option<VfxTextureMultDefinitionData>,
) -> Option<ConfigVfxTextureMult> {
    mult.as_ref().map(|m| ConfigVfxTextureMult {
        texture_mult: m.texture_mult.clone(),
    })
}

fn convert_material_override(
    overrides: &Option<Vec<VfxMaterialOverrideDefinitionData>>,
) -> Option<Vec<ConfigVfxMaterialOverride>> {
    overrides.as_ref().map(|list| {
        list.iter()
            .map(|o| ConfigVfxMaterialOverride {
                base_texture: o.base_texture.clone(),
            })
            .collect()
    })
}

pub fn convert_emitter(def: &VfxEmitterDefinitionData) -> ConfigVfxEmitterDefinition {
    ConfigVfxEmitterDefinition {
        emitter_name: def.emitter_name.clone(),
        lifetime: def.lifetime,
        birth_acceleration: convert_stochastic_vector3(&def.birth_acceleration, Vec3::ZERO),
        birth_color: convert_stochastic_color(&def.birth_color, Vec4::ONE),
        birth_rotation0: convert_stochastic_vector3(&def.birth_rotation0, Vec3::ZERO),
        birth_scale0: convert_stochastic_vector3(&def.birth_scale0, Vec3::ONE),
        birth_uv_offset: convert_stochastic_vector2(&def.birth_uv_offset, Vec2::ZERO),
        birth_uv_scroll_rate: convert_stochastic_vector2(&def.birth_uv_scroll_rate, Vec2::ZERO),
        birth_velocity: convert_stochastic_vector3(&def.birth_velocity, Vec3::ZERO),
        bind_weight: convert_stochastic_float(&def.bind_weight, 0.0),
        color: convert_stochastic_color(&def.color, Vec4::ONE),
        scale0: convert_stochastic_vector3(&def.scale0, Vec3::ONE),
        particle_lifetime: convert_stochastic_float(&def.particle_lifetime, 1.0),
        rate: convert_stochastic_float(&def.rate, 1.0),
        emitter_position: convert_stochastic_vector3(&def.emitter_position, Vec3::ZERO),

        distortion_definition: convert_distortion(&def.distortion_definition),
        num_frames: def.num_frames,
        blend_mode: def.blend_mode,
        material_override_definitions: convert_material_override(
            &def.material_override_definitions,
        ),
        primitive: convert_primitive(&def.primitive),
        is_single_particle: def.is_single_particle,
        is_uniform_scale: def.is_uniform_scale,
        is_random_start_frame: def.is_random_start_frame,
        is_local_orientation: def.is_local_orientation,
        texture: def.texture.clone(),
        particle_color_texture: def.particle_color_texture.clone(),
        tex_div: def.tex_div,
        slice_technique_range: def.slice_technique_range,
        texture_mult: convert_texture_mult(&def.texture_mult),
        alpha_ref: def.alpha_ref,
        spawn_shape: convert_shape(&def.spawn_shape),
    }
}

pub fn convert_system_definition(def: &VfxSystemDefinitionData) -> ConfigVfxSystemDefinition {
    let complex = def
        .complex_emitter_definition_data
        .as_ref()
        .map(|list| list.iter().map(convert_emitter).collect());
    let simple = def
        .simple_emitter_definition_data
        .as_ref()
        .map(|list| list.iter().map(convert_emitter).collect());
    ConfigVfxSystemDefinition {
        particle_name: def.particle_name.clone(),
        particle_path: def.particle_path.clone(),
        complex_emitter_definition_data: complex,
        simple_emitter_definition_data: simple,
    }
}
