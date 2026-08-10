//! 工作副本访问 + 发射器字段编辑（数值/标志/贴图/采样器）。

use lol_share::{
    ConfigVfxEmitterDefinition, ConfigVfxSystemDefinition, Sampler, StochasticSampler, VfxTexture,
};

use super::state::update_state;

// ── 工作副本访问 ──

/// 取「主发射器列表」：complex 非空用 complex，否则用 simple。
pub(super) fn primary_list_ref(
    wd: &ConfigVfxSystemDefinition,
) -> Option<&Vec<ConfigVfxEmitterDefinition>> {
    if let Some(l) = wd.complex_emitter_definition_data.as_ref() {
        if !l.is_empty() {
            return Some(l);
        }
    }
    wd.simple_emitter_definition_data.as_ref()
}

pub(super) fn primary_list_mut(
    wd: &mut ConfigVfxSystemDefinition,
) -> Option<&mut Vec<ConfigVfxEmitterDefinition>> {
    if let Some(l) = wd.complex_emitter_definition_data.as_mut() {
        if !l.is_empty() {
            return Some(l);
        }
    }
    wd.simple_emitter_definition_data.as_mut()
}

pub(super) fn mutate_emitter(idx: usize, f: impl FnOnce(&mut ConfigVfxEmitterDefinition)) {
    update_state(|s| {
        let Some(wd) = &mut s.working_def else {
            return;
        };
        if let Some(list) = primary_list_mut(wd) {
            if let Some(em) = list.get_mut(idx) {
                f(em);
            }
        }
    });
}

// ── 发射器字段编辑 ──

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum NumField {
    Lifetime,
    NumFrames,
    BlendMode,
    AlphaRef,
}

pub(super) fn get_num_field(em: &ConfigVfxEmitterDefinition, field: NumField) -> f32 {
    match field {
        NumField::Lifetime => em.lifetime.unwrap_or(0.0),
        NumField::NumFrames => em.num_frames.map(|v| v as f32).unwrap_or(1.0),
        NumField::BlendMode => em.blend_mode.map(|v| v as f32).unwrap_or(0.0),
        NumField::AlphaRef => em.alpha_ref.map(|v| v as f32).unwrap_or(0.0),
    }
}

pub(super) fn set_num_field(idx: usize, field: NumField, v: f32) {
    mutate_emitter(idx, |em| match field {
        NumField::Lifetime => em.lifetime = Some(v),
        NumField::NumFrames => em.num_frames = Some(v.max(1.0) as u16),
        NumField::BlendMode => em.blend_mode = Some(v.clamp(0.0, 255.0) as u8),
        NumField::AlphaRef => em.alpha_ref = Some(v.clamp(0.0, 255.0) as u8),
    });
}

pub(super) fn set_name_idx(idx: usize, name: String) {
    mutate_emitter(idx, |em| {
        let trimmed = name.trim().to_string();
        em.emitter_name = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        };
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FlagField {
    IsSingleParticle,
    IsUniformScale,
    IsRandomStartFrame,
    IsLocalOrientation,
    IsDirectionOriented,
    SoftParticle,
}

pub(super) const FLAGS: &[(FlagField, &str)] = &[
    (FlagField::IsSingleParticle, "单粒子"),
    (FlagField::IsUniformScale, "等比缩放"),
    (FlagField::IsRandomStartFrame, "随机起始帧"),
    (FlagField::IsLocalOrientation, "局部朝向"),
    (FlagField::IsDirectionOriented, "方向对齐"),
    (FlagField::SoftParticle, "软粒子"),
];

pub(super) fn get_flag(em: &ConfigVfxEmitterDefinition, flag: FlagField) -> bool {
    match flag {
        FlagField::IsSingleParticle => em.is_single_particle.unwrap_or(false),
        FlagField::IsUniformScale => em.is_uniform_scale.unwrap_or(false),
        FlagField::IsRandomStartFrame => em.is_random_start_frame.unwrap_or(false),
        FlagField::IsLocalOrientation => em.is_local_orientation.unwrap_or(false),
        FlagField::IsDirectionOriented => em.is_direction_oriented.unwrap_or(false),
        FlagField::SoftParticle => em.soft_particle_definition.unwrap_or(false),
    }
}

fn set_flag(em: &mut ConfigVfxEmitterDefinition, flag: FlagField, on: bool) {
    let f = Some(on);
    match flag {
        FlagField::IsSingleParticle => em.is_single_particle = f,
        FlagField::IsUniformScale => em.is_uniform_scale = f,
        FlagField::IsRandomStartFrame => em.is_random_start_frame = f,
        FlagField::IsLocalOrientation => em.is_local_orientation = f,
        FlagField::IsDirectionOriented => em.is_direction_oriented = f,
        FlagField::SoftParticle => em.soft_particle_definition = f,
    }
}

pub(super) fn set_flag_idx(idx: usize, flag: FlagField, on: bool) {
    mutate_emitter(idx, |em| set_flag(em, flag, on));
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum TexField {
    Texture,
    ParticleColorTexture,
    Palette,
    Reflection,
}

pub(super) const TEX_ITEMS: &[(TexField, &str, &str)] = &[
    (
        TexField::Texture,
        "主贴图 texture",
        "ASSETS/Textures/particles/fire.dds",
    ),
    (
        TexField::ParticleColorTexture,
        "粒子颜色贴图 particle_color_texture",
        "ASSETS/Textures/particles/color.dds",
    ),
    (
        TexField::Palette,
        "调色板 palette_definition",
        "ASSETS/Textures/particles/palette.dds",
    ),
    (
        TexField::Reflection,
        "反射贴图 reflection_definition",
        "ASSETS/Textures/particles/reflection.dds",
    ),
];

pub(super) fn get_texture(em: &ConfigVfxEmitterDefinition, f: TexField) -> String {
    match f {
        TexField::Texture => em
            .texture
            .as_ref()
            .map(|t| t.path.clone())
            .unwrap_or_default(),
        TexField::ParticleColorTexture => em
            .particle_color_texture
            .as_ref()
            .map(|t| t.path.clone())
            .unwrap_or_default(),
        TexField::Palette => em
            .palette_definition
            .as_ref()
            .map(|t| t.path.clone())
            .unwrap_or_default(),
        TexField::Reflection => em
            .reflection_definition
            .as_ref()
            .map(|t| t.path.clone())
            .unwrap_or_default(),
    }
}

fn set_texture(em: &mut ConfigVfxEmitterDefinition, f: TexField, path: String) {
    let tex = if path.trim().is_empty() {
        None
    } else {
        Some(VfxTexture::from_path(path.trim().to_string()))
    };
    match f {
        TexField::Texture => em.texture = tex,
        TexField::ParticleColorTexture => em.particle_color_texture = tex,
        TexField::Palette => em.palette_definition = tex,
        TexField::Reflection => em.reflection_definition = tex,
    }
}

pub(super) fn set_texture_idx(idx: usize, f: TexField, path: String) {
    mutate_emitter(idx, |em| set_texture(em, f, path));
}

pub(super) fn tex_div_values(em: &ConfigVfxEmitterDefinition) -> [f32; 2] {
    em.tex_div.map(|v| v.to_array()).unwrap_or([1.0, 1.0])
}

pub(super) fn set_tex_div_comp(idx: usize, comp: usize, v: f32) {
    mutate_emitter(idx, |em| {
        let mut vals = tex_div_values(em);
        if comp < 2 {
            vals[comp] = v;
        }
        match &mut em.tex_div {
            Some(vv) => {
                vv.x = vals[0];
                vv.y = vals[1];
            }
            None => em.tex_div = Some(vals.into()),
        }
    });
}

// ── 采样器（StochasticSampler）编辑 ──
//
// 简化：常量模式直接编辑常量值；曲线模式编辑首采样点值；通过「常量/曲线」下拉切换，
// 不实现 SVG 拖拽曲线。数值写入只改 base_sampler，prob_curves 原样保留。

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum SamplerKind {
    Rate,
    ParticleLifetime,
    BindWeight,
    EmitterPosition,
    BirthVelocity,
    BirthAcceleration,
    BirthRotation0,
    BirthScale0,
    Scale0,
    BirthColor,
    Color,
    BirthUvOffset,
    BirthUvScrollRate,
}

impl SamplerKind {
    pub(super) fn label(&self) -> &'static str {
        match self {
            SamplerKind::Rate => "生成速率 rate",
            SamplerKind::ParticleLifetime => "粒子寿命 particle_lifetime",
            SamplerKind::BindWeight => "绑定权重 bind_weight",
            SamplerKind::EmitterPosition => "发射器位置 emitter_position",
            SamplerKind::BirthVelocity => "出生速度 birth_velocity",
            SamplerKind::BirthAcceleration => "出生加速度 birth_acceleration",
            SamplerKind::BirthRotation0 => "出生旋转 birth_rotation0",
            SamplerKind::BirthScale0 => "出生缩放 birth_scale0",
            SamplerKind::Scale0 => "缩放 scale0",
            SamplerKind::BirthColor => "出生颜色 birth_color",
            SamplerKind::Color => "颜色 color",
            SamplerKind::BirthUvOffset => "出生UV偏移 birth_uv_offset",
            SamplerKind::BirthUvScrollRate => "出生UV滚动 birth_uv_scroll_rate",
        }
    }

    pub(super) fn dims(&self) -> usize {
        match self {
            SamplerKind::Rate | SamplerKind::ParticleLifetime | SamplerKind::BindWeight => 1,
            SamplerKind::BirthUvOffset | SamplerKind::BirthUvScrollRate => 2,
            SamplerKind::BirthColor | SamplerKind::Color => 4,
            _ => 3,
        }
    }

    pub(super) fn all() -> [SamplerKind; 13] {
        [
            SamplerKind::Rate,
            SamplerKind::ParticleLifetime,
            SamplerKind::BindWeight,
            SamplerKind::EmitterPosition,
            SamplerKind::BirthVelocity,
            SamplerKind::BirthAcceleration,
            SamplerKind::BirthRotation0,
            SamplerKind::BirthScale0,
            SamplerKind::Scale0,
            SamplerKind::BirthColor,
            SamplerKind::Color,
            SamplerKind::BirthUvOffset,
            SamplerKind::BirthUvScrollRate,
        ]
    }
}

macro_rules! read_scalar_sampler {
    ($s:expr) => {{
        let curve = matches!(&$s.base_sampler, Sampler::Curve { .. });
        let v = match &$s.base_sampler {
            Sampler::Constant(v) => *v,
            Sampler::Curve { samples } => samples.first().map(|(_, v)| *v).unwrap_or(0.0),
        };
        (vec![v], curve)
    }};
}

macro_rules! read_vec_sampler {
    ($s:expr, $dims:expr) => {{
        let curve = matches!(&$s.base_sampler, Sampler::Curve { .. });
        let vals = match &$s.base_sampler {
            Sampler::Constant(v) => v.to_array().to_vec(),
            Sampler::Curve { samples } => samples
                .first()
                .map(|(_, v)| v.to_array().to_vec())
                .unwrap_or_else(|| vec![0.0; $dims]),
        };
        (vals, curve)
    }};
}

/// 读取采样器当前值（常量或曲线首点）与是否为曲线模式。
pub(super) fn read_sampler(em: &ConfigVfxEmitterDefinition, kind: SamplerKind) -> (Vec<f32>, bool) {
    match kind {
        SamplerKind::Rate => read_scalar_sampler!(em.rate),
        SamplerKind::ParticleLifetime => read_scalar_sampler!(em.particle_lifetime),
        SamplerKind::BindWeight => read_scalar_sampler!(em.bind_weight),
        SamplerKind::EmitterPosition => read_vec_sampler!(em.emitter_position, 3),
        SamplerKind::BirthVelocity => read_vec_sampler!(em.birth_velocity, 3),
        SamplerKind::BirthAcceleration => read_vec_sampler!(em.birth_acceleration, 3),
        SamplerKind::BirthRotation0 => read_vec_sampler!(em.birth_rotation0, 3),
        SamplerKind::BirthScale0 => read_vec_sampler!(em.birth_scale0, 3),
        SamplerKind::Scale0 => read_vec_sampler!(em.scale0, 3),
        SamplerKind::BirthColor => read_vec_sampler!(em.birth_color, 4),
        SamplerKind::Color => read_vec_sampler!(em.color, 4),
        SamplerKind::BirthUvOffset => read_vec_sampler!(em.birth_uv_offset, 2),
        SamplerKind::BirthUvScrollRate => read_vec_sampler!(em.birth_uv_scroll_rate, 2),
    }
}

macro_rules! write_scalar_sampler {
    ($s:expr, $v:expr) => {{
        let v = $v;
        match &mut $s.base_sampler {
            Sampler::Constant(c) => *c = v,
            Sampler::Curve { samples } => {
                if let Some((_, c)) = samples.first_mut() {
                    *c = v;
                }
            }
        }
    }};
}

macro_rules! write_vec_sampler {
    ($s:expr, $vals:expr, [$($i:expr),*]) => {{
        let vals = $vals;
        match &mut $s.base_sampler {
            Sampler::Constant(v) => {
                let mut arr = v.to_array();
                $( arr[$i] = vals[$i]; )*
                *v = arr.into();
            }
            Sampler::Curve { samples } => {
                if let Some((_, v)) = samples.first_mut() {
                    let mut arr = v.to_array();
                    $( arr[$i] = vals[$i]; )*
                    *v = arr.into();
                }
            }
        }
    }};
}

/// 回写采样器整组数值（常量或曲线首点）。
fn write_sampler_values(em: &mut ConfigVfxEmitterDefinition, kind: SamplerKind, vals: Vec<f32>) {
    match kind {
        SamplerKind::Rate => write_scalar_sampler!(em.rate, vals[0]),
        SamplerKind::ParticleLifetime => write_scalar_sampler!(em.particle_lifetime, vals[0]),
        SamplerKind::BindWeight => write_scalar_sampler!(em.bind_weight, vals[0]),
        SamplerKind::EmitterPosition => write_vec_sampler!(em.emitter_position, vals, [0, 1, 2]),
        SamplerKind::BirthVelocity => write_vec_sampler!(em.birth_velocity, vals, [0, 1, 2]),
        SamplerKind::BirthAcceleration => {
            write_vec_sampler!(em.birth_acceleration, vals, [0, 1, 2])
        }
        SamplerKind::BirthRotation0 => write_vec_sampler!(em.birth_rotation0, vals, [0, 1, 2]),
        SamplerKind::BirthScale0 => write_vec_sampler!(em.birth_scale0, vals, [0, 1, 2]),
        SamplerKind::Scale0 => write_vec_sampler!(em.scale0, vals, [0, 1, 2]),
        SamplerKind::BirthColor => write_vec_sampler!(em.birth_color, vals, [0, 1, 2, 3]),
        SamplerKind::Color => write_vec_sampler!(em.color, vals, [0, 1, 2, 3]),
        SamplerKind::BirthUvOffset => write_vec_sampler!(em.birth_uv_offset, vals, [0, 1]),
        SamplerKind::BirthUvScrollRate => write_vec_sampler!(em.birth_uv_scroll_rate, vals, [0, 1]),
    }
}

pub(super) fn set_sampler_component(idx: usize, kind: SamplerKind, comp: usize, v: f32) {
    mutate_emitter(idx, |em| {
        let (mut vals, _) = read_sampler(em, kind);
        if comp < vals.len() {
            vals[comp] = v;
        }
        write_sampler_values(em, kind, vals);
    });
}

/// 通用模式切换：常量 ↔ 曲线（2 个关键帧），需要 T: Clone，不依赖具体向量类型。
fn set_mode<T: Clone>(s: &mut StochasticSampler<T>, curve: bool) {
    let cur = match &s.base_sampler {
        Sampler::Constant(v) => Some(v.clone()),
        Sampler::Curve { samples } => samples.first().map(|(_, v)| v.clone()),
    };
    let Some(v) = cur else {
        return;
    };
    if curve {
        if matches!(&s.base_sampler, Sampler::Constant(_)) {
            s.base_sampler = Sampler::Curve {
                samples: vec![(0.0, v.clone()), (1.0, v)],
            };
        }
    } else if matches!(&s.base_sampler, Sampler::Curve { .. }) {
        s.base_sampler = Sampler::Constant(v);
    }
}

fn set_sampler_mode(em: &mut ConfigVfxEmitterDefinition, kind: SamplerKind, curve: bool) {
    match kind {
        SamplerKind::Rate => set_mode(&mut em.rate, curve),
        SamplerKind::ParticleLifetime => set_mode(&mut em.particle_lifetime, curve),
        SamplerKind::BindWeight => set_mode(&mut em.bind_weight, curve),
        SamplerKind::EmitterPosition => set_mode(&mut em.emitter_position, curve),
        SamplerKind::BirthVelocity => set_mode(&mut em.birth_velocity, curve),
        SamplerKind::BirthAcceleration => set_mode(&mut em.birth_acceleration, curve),
        SamplerKind::BirthRotation0 => set_mode(&mut em.birth_rotation0, curve),
        SamplerKind::BirthScale0 => set_mode(&mut em.birth_scale0, curve),
        SamplerKind::Scale0 => set_mode(&mut em.scale0, curve),
        SamplerKind::BirthColor => set_mode(&mut em.birth_color, curve),
        SamplerKind::Color => set_mode(&mut em.color, curve),
        SamplerKind::BirthUvOffset => set_mode(&mut em.birth_uv_offset, curve),
        SamplerKind::BirthUvScrollRate => set_mode(&mut em.birth_uv_scroll_rate, curve),
    }
}

pub(super) fn set_sampler_mode_idx(idx: usize, kind: SamplerKind, curve: bool) {
    mutate_emitter(idx, |em| set_sampler_mode(em, kind, curve));
}
