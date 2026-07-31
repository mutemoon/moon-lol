//! 粒子系统的纯 serde 数据模型（ConfigVfx 家族）。
//!
//! 因为桌面端后端只需要「解析 skin{N}_vfx.ron + 重新序列化每个 system 为 RON」
//! 发给粒子渲染 server，所以本 crate 只含落盘字段，不依赖 bevy。
//! lol_base_render 有自己独立的运行时定义（带 Handle/curve/Asset），
//! 字段名一致即 RON 往返无损。

use std::collections::BTreeMap;

use glam::{Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Sampler / ProbabilityCurve：磁盘上只区分 Constant(T) 与 Curve(Vec<(f32, T)>)。
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum Sampler<T> {
    Constant(T),
    Curve { samples: Vec<(f32, T)> },
}

impl<T: Serialize + Clone> Serialize for Sampler<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        #[serde(untagged)]
        enum Helper<U> {
            Constant(U),
            Curve(Vec<(f32, U)>),
        }
        match self {
            Self::Constant(v) => Helper::Constant(v.clone()).serialize(serializer),
            Self::Curve { samples } => Helper::Curve(samples.clone()).serialize(serializer),
        }
    }
}

impl<'de, T: Deserialize<'de> + Clone> Deserialize<'de> for Sampler<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Helper<U> {
            Constant(U),
            Curve(Vec<(f32, U)>),
        }
        let helper = Helper::<T>::deserialize(deserializer)?;
        Ok(match helper {
            Helper::Constant(v) => Self::Constant(v),
            Helper::Curve(samples) => Self::Curve { samples },
        })
    }
}

#[derive(Clone, Debug)]
pub enum ProbabilityCurve {
    Constant(f32),
    Curve { samples: Vec<(f32, f32)> },
}

impl Serialize for ProbabilityCurve {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        #[serde(untagged)]
        enum Helper {
            Constant(f32),
            Curve(Vec<(f32, f32)>),
        }
        match self {
            Self::Constant(v) => Helper::Constant(*v).serialize(serializer),
            Self::Curve { samples } => Helper::Curve(samples.clone()).serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ProbabilityCurve {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Helper {
            Constant(f32),
            Curve(Vec<(f32, f32)>),
        }
        let helper = Helper::deserialize(deserializer)?;
        Ok(match helper {
            Helper::Constant(v) => Self::Constant(v),
            Helper::Curve(samples) => Self::Curve { samples },
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize + Clone",
    deserialize = "T: Deserialize<'de> + Clone"
))]
pub struct StochasticSampler<T> {
    pub base_sampler: Sampler<T>,
    pub prob_curves: Vec<Option<ProbabilityCurve>>,
}

// ---------------------------------------------------------------------------
// VfxTexture：磁盘只序列化 path 字符串，handle 由 lol_base_render 运行时回填。
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct VfxTexture {
    pub path: String,
}

impl VfxTexture {
    pub fn from_path(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl Serialize for VfxTexture {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.path.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for VfxTexture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let path = String::deserialize(deserializer)?;
        Ok(Self { path })
    }
}

// ---------------------------------------------------------------------------
// ConfigVfx 家族
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub texture: Option<VfxTexture>,
    pub particle_color_texture: Option<VfxTexture>,
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
    pub normal_map_texture: Option<VfxTexture>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigVfxMaterialOverride {
    pub base_texture: Option<VfxTexture>,
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
    pub texture_mult: Option<VfxTexture>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigResourceResolver {
    pub resource_map: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigVfx {
    pub resolvers: BTreeMap<u32, ConfigResourceResolver>,
    pub systems: BTreeMap<u32, ConfigVfxSystemDefinition>,
}
