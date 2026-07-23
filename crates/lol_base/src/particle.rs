use std::collections::BTreeMap;

use bevy::math::StableInterpolate;
use bevy::math::curve::{Curve, Interval, UnevenSampleAutoCurve};
use bevy::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// Custom serialization/deserialization helper
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum SerializedSampler<T> {
    Constant(T),
    Curve(Vec<(f32, T)>),
}

#[derive(Clone, Debug)]
pub enum Sampler<T> {
    Constant(T),
    Curve {
        samples: Vec<(f32, T)>,
        curve: UnevenSampleAutoCurve<T>,
    },
}

impl<T: Clone> Sampler<T> {
    pub fn new_curve(samples: Vec<(f32, T)>) -> Result<Self, String>
    where
        T: StableInterpolate,
    {
        let curve = UnevenSampleAutoCurve::new(samples.clone()).map_err(|e| format!("{:?}", e))?;
        Ok(Self::Curve { samples, curve })
    }
}

impl<T: Serialize + Clone> Serialize for Sampler<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Constant(v) => SerializedSampler::Constant(v.clone()).serialize(serializer),
            Self::Curve { samples, .. } => {
                SerializedSampler::Curve(samples.clone()).serialize(serializer)
            }
        }
    }
}

impl<'de, T> Deserialize<'de> for Sampler<T>
where
    T: Deserialize<'de> + StableInterpolate + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = SerializedSampler::<T>::deserialize(deserializer)?;
        match helper {
            SerializedSampler::Constant(v) => Ok(Self::Constant(v)),
            SerializedSampler::Curve(samples) => {
                let curve = UnevenSampleAutoCurve::new(samples.clone())
                    .map_err(serde::de::Error::custom)?;
                Ok(Self::Curve { samples, curve })
            }
        }
    }
}

impl<T: StableInterpolate + Clone + Copy> Curve<T> for Sampler<T> {
    fn sample_clamped(&self, t: f32) -> T {
        match self {
            Self::Constant(v) => *v,
            Self::Curve { curve, .. } => curve.sample_clamped(t),
        }
    }

    fn sample_unchecked(&self, t: f32) -> T {
        match self {
            Self::Constant(v) => *v,
            Self::Curve { curve, .. } => curve.sample_unchecked(t),
        }
    }

    fn domain(&self) -> Interval {
        Interval::EVERYWHERE
    }
}

#[derive(Clone, Debug)]
pub enum ProbabilityCurve {
    Constant(f32),
    Curve {
        samples: Vec<(f32, f32)>,
        curve: UnevenSampleAutoCurve<f32>,
    },
}

impl ProbabilityCurve {
    pub fn new_curve(samples: Vec<(f32, f32)>) -> Result<Self, String> {
        let curve = UnevenSampleAutoCurve::new(samples.clone()).map_err(|e| format!("{:?}", e))?;
        Ok(Self::Curve { samples, curve })
    }

    pub fn sample(&self, rng: &mut impl rand::Rng) -> f32 {
        match self {
            Self::Constant(v) => *v,
            Self::Curve { curve, .. } => {
                let t = rng.random_range(0.0..=1.0);
                curve.sample_clamped(t)
            }
        }
    }
}

impl Serialize for ProbabilityCurve {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Constant(v) => SerializedSampler::Constant(*v).serialize(serializer),
            Self::Curve { samples, .. } => {
                SerializedSampler::Curve(samples.clone()).serialize(serializer)
            }
        }
    }
}

impl<'de> Deserialize<'de> for ProbabilityCurve {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = SerializedSampler::<f32>::deserialize(deserializer)?;
        match helper {
            SerializedSampler::Constant(v) => Ok(Self::Constant(v)),
            SerializedSampler::Curve(samples) => {
                let curve = UnevenSampleAutoCurve::new(samples.clone())
                    .map_err(serde::de::Error::custom)?;
                Ok(Self::Curve { samples, curve })
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize + Clone",
    deserialize = "T: Deserialize<'de> + StableInterpolate + Clone"
))]
pub struct StochasticSampler<T> {
    pub base_sampler: Sampler<T>,
    pub prob_curves: Vec<Option<ProbabilityCurve>>,
}

pub trait CombineMultiplicative {
    fn combine_mul(&self, components: &[f32]) -> Self;
}

impl CombineMultiplicative for f32 {
    fn combine_mul(&self, components: &[f32]) -> Self {
        let rand_multiplier = components.get(0).unwrap_or(&1.0);
        self * rand_multiplier
    }
}

impl CombineMultiplicative for Vec2 {
    fn combine_mul(&self, components: &[f32]) -> Self {
        let rand_x = components.get(0).unwrap_or(&1.0);
        let rand_y = components.get(1).unwrap_or(&1.0);
        Vec2 {
            x: self.x * rand_x,
            y: self.y * rand_y,
        }
    }
}

impl CombineMultiplicative for Vec3 {
    fn combine_mul(&self, components: &[f32]) -> Self {
        let rand_x = components.get(0).unwrap_or(&1.0);
        let rand_y = components.get(1).unwrap_or(&1.0);
        let rand_z = components.get(2).unwrap_or(&1.0);
        Vec3 {
            x: self.x * rand_x,
            y: self.y * rand_y,
            z: self.z * rand_z,
        }
    }
}

impl CombineMultiplicative for Vec4 {
    fn combine_mul(&self, components: &[f32]) -> Self {
        let rand_x = components.get(0).unwrap_or(&1.0);
        let rand_y = components.get(1).unwrap_or(&1.0);
        let rand_z = components.get(2).unwrap_or(&1.0);
        let rand_w = components.get(3).unwrap_or(&1.0);
        Vec4::new(
            self.x * rand_x,
            self.y * rand_y,
            self.z * rand_z,
            self.w * rand_w,
        )
    }
}

impl<T> StochasticSampler<T> {
    pub fn sample_clamped(&self, t: f32) -> T
    where
        T: CombineMultiplicative + Copy + 'static,
        Sampler<T>: Curve<T>,
    {
        let mut rng = rand::rng();
        let base_value = self.base_sampler.sample_clamped(t);
        let random_components: Vec<f32> = self
            .prob_curves
            .iter()
            .map(|opt_curve| {
                opt_curve
                    .as_ref()
                    .map_or(1.0, |curve| curve.sample(&mut rng))
            })
            .collect();
        base_value.combine_mul(&random_components)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Asset, TypePath)]
pub struct ConfigVfxSystemDefinition {
    pub particle_name: String,
    pub particle_path: String,
    pub complex_emitter_definition_data: Option<Vec<ConfigVfxEmitterDefinition>>,
    pub simple_emitter_definition_data: Option<Vec<ConfigVfxEmitterDefinition>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigVfxEmitterDefinition {
    pub emitter_name: Option<String>,
    pub lifetime: Option<f32>,
    pub birth_acceleration: StochasticSampler<Vec3>,
    pub birth_color: StochasticSampler<Vec4>,
    pub birth_rotation0: StochasticSampler<Vec3>,
    pub birth_scale0: StochasticSampler<Vec3>,
    pub birth_uv_offset: StochasticSampler<Vec2>,
    pub birth_uv_scroll_rate: StochasticSampler<Vec2>,
    pub birth_velocity: StochasticSampler<Vec3>,
    pub bind_weight: StochasticSampler<f32>,
    pub color: StochasticSampler<Vec4>,
    pub scale0: StochasticSampler<Vec3>,
    pub particle_lifetime: StochasticSampler<f32>,
    pub rate: StochasticSampler<f32>,
    pub emitter_position: StochasticSampler<Vec3>,

    pub distortion_definition: Option<ConfigVfxDistortionDefinition>,
    pub num_frames: Option<u16>,
    pub blend_mode: Option<u8>,
    pub material_override_definitions: Option<Vec<ConfigVfxMaterialOverride>>,
    pub primitive: Option<ConfigVfxPrimitive>,
    pub is_single_particle: Option<bool>,
    pub is_uniform_scale: Option<bool>,
    pub is_random_start_frame: Option<bool>,
    pub is_local_orientation: Option<bool>,
    pub texture: Option<String>,
    pub particle_color_texture: Option<String>,
    pub tex_div: Option<Vec2>,
    pub slice_technique_range: Option<f32>,
    pub texture_mult: Option<ConfigVfxTextureMult>,
    pub alpha_ref: Option<u8>,
    pub spawn_shape: Option<ConfigVfxShape>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigVfxDistortionDefinition {
    pub distortion: Option<f32>,
    pub distortion_mode: Option<u8>,
    pub normal_map_texture: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigVfxMaterialOverride {
    pub base_texture: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ConfigVfxPrimitive {
    Unk0x8df5fcf7,
    VfxPrimitiveArbitraryQuad,
    VfxPrimitiveArbitraryTrail,
    VfxPrimitiveAttachedMesh {
        align_pitch_to_camera: Option<bool>,
        align_yaw_to_camera: Option<bool>,
        simple_mesh_name: Option<String>,
    },
    VfxPrimitiveBeam,
    VfxPrimitiveCameraSegmentBeam,
    VfxPrimitiveCameraTrail,
    VfxPrimitiveCameraUnitQuad,
    VfxPrimitiveMesh {
        align_pitch_to_camera: Option<bool>,
        align_yaw_to_camera: Option<bool>,
        simple_mesh_name: Option<String>,
    },
    VfxPrimitivePlanarProjection {
        y_range: Option<f32>,
    },
    VfxPrimitiveRay,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigVfxTextureMult {
    pub texture_mult: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigVfxShape {
    Box {
        flags: Option<u8>,
        size: Option<Vec3>,
    },
    Cylinder {
        flags: Option<u8>,
        height: Option<f32>,
        radius: Option<f32>,
    },
    Legacy {
        emit_offset: StochasticSampler<Vec3>,
        emit_rotation_angles: Vec<StochasticSampler<f32>>,
        emit_rotation_axes: Vec<Vec3>,
    },
    Sphere {
        flags: Option<u8>,
        radius: Option<f32>,
    },
    Unk0xee39916f {
        emit_offset: Option<Vec3>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, Asset, TypePath)]
pub struct ConfigResourceResolver {
    pub resource_map: BTreeMap<u32, u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Asset, TypePath)]
pub struct ConfigVfx {
    pub resolvers: BTreeMap<u32, ConfigResourceResolver>,
    pub systems: BTreeMap<u32, ConfigVfxSystemDefinition>,
}
