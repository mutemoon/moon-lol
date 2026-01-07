use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use league_core::{
    ValueColor, ValueFloat, ValueVector2, ValueVector3, VfxAnimatedColorVariableData,
    VfxProbabilityTableData,
};
use rand::Rng;

pub fn create_black_pixel_texture() -> Image {
    let image = Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8Unorm,
        default(),
    );

    image
}

#[derive(Debug, Clone)]
pub enum Sampler<T> {
    Constant(T),
    Curve(UnevenSampleAutoCurve<T>),
}

macro_rules! impl_sampler_traits {
    ($Sampler:ty, $Value:ty, $Data:ty, $domain_logic:expr) => {
        impl Curve<$Data> for $Sampler {
            fn sample_clamped(&self, t: f32) -> $Data {
                match self {
                    Self::Constant(v) => *v,
                    Self::Curve(c) => c.sample_clamped(t),
                }
            }

            fn sample_unchecked(&self, t: f32) -> $Data {
                match self {
                    Self::Constant(v) => *v,
                    Self::Curve(c) => c.sample_unchecked(t),
                }
            }

            fn domain(&self) -> Interval {
                $domain_logic
            }
        }

        impl From<$Value> for $Sampler {
            fn from(value: $Value) -> Self {
                if let Some(dynamics) = value.dynamics {
                    match dynamics.times.len() {
                        0 => Self::Constant(value.constant_value.unwrap_or_default()),
                        1 => Self::Constant(dynamics.values.into_iter().next().unwrap()),
                        _ => {
                            let samples = dynamics.times.into_iter().zip(dynamics.values);
                            match UnevenSampleAutoCurve::new(samples) {
                                Ok(curve) => Self::Curve(curve),
                                Err(_) => Self::Constant(value.constant_value.unwrap_or_default()),
                            }
                        }
                    }
                } else {
                    Self::Constant(value.constant_value.unwrap_or_default())
                }
            }
        }
    };
}

impl_sampler_traits!(Sampler<f32>, ValueFloat, f32, Interval::EVERYWHERE);

impl_sampler_traits!(Sampler<Vec3>, ValueVector3, Vec3, Interval::EVERYWHERE);

impl_sampler_traits!(Sampler<Vec2>, ValueVector2, Vec2, Interval::EVERYWHERE);

impl Curve<Vec4> for Sampler<Vec4> {
    fn sample_clamped(&self, t: f32) -> Vec4 {
        match self {
            Self::Constant(v) => *v,
            Self::Curve(c) => c.sample_clamped(t),
        }
    }
    fn sample_unchecked(&self, t: f32) -> Vec4 {
        match self {
            Self::Constant(v) => *v,
            Self::Curve(c) => c.sample_unchecked(t),
        }
    }
    fn domain(&self) -> Interval {
        Interval::EVERYWHERE
    }
}

impl From<ValueColor> for Sampler<Vec4> {
    fn from(value: ValueColor) -> Self {
        if let Some(VfxAnimatedColorVariableData {
            times: Some(times),
            values: Some(values),
            ..
        }) = value.dynamics
        {
            match times.len() {
                0 => Self::Constant(value.constant_value.unwrap_or_default()),
                1 => Self::Constant(values.into_iter().next().unwrap()),
                _ => {
                    let samples = times.into_iter().zip(values);
                    match UnevenSampleAutoCurve::new(samples) {
                        Ok(curve) => Self::Curve(curve),
                        Err(_) => Self::Constant(value.constant_value.unwrap_or_default()),
                    }
                }
            }
        } else {
            Self::Constant(value.constant_value.unwrap_or_default())
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProbabilityCurve {
    Constant(f32),
    Curve(UnevenSampleAutoCurve<f32>),
}

impl ProbabilityCurve {
    pub fn new(table: VfxProbabilityTableData) -> Option<Self> {
        let (times, values) = match (table.key_times, table.key_values) {
            (Some(times), Some(values)) => (times, values),
            _ => return None,
        };

        if times.len() != values.len() {
            return None;
        }

        match times.len() {
            0 => None,
            1 => Some(Self::Constant(values[0])),
            _ => {
                let samples = times.into_iter().zip(values);

                UnevenSampleAutoCurve::new(samples).ok().map(Self::Curve)
            }
        }
    }

    pub fn sample(&self, rng: &mut impl Rng) -> f32 {
        match self {
            Self::Constant(v) => *v,

            Self::Curve(c) => {
                let t = rng.random_range(0.0..=1.0);
                c.sample_clamped(t)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct StochasticSampler<T> {
    pub base_sampler: Sampler<T>,

    pub prob_curves: Vec<Option<ProbabilityCurve>>,
}

impl From<ValueFloat> for StochasticSampler<f32> {
    fn from(value: ValueFloat) -> Self {
        let base_sampler = Sampler::<f32>::from(value.clone());

        let mut prob_curves = Vec::new();

        if let Some(dynamics) = value.dynamics.as_ref() {
            if let Some(tables) = dynamics.probability_tables.as_ref() {
                let curve_opt = tables
                    .iter()
                    .next()
                    .cloned()
                    .and_then(ProbabilityCurve::new);
                prob_curves.push(curve_opt);
            }
        }

        Self {
            base_sampler,
            prob_curves,
        }
    }
}

impl From<ValueVector3> for StochasticSampler<Vec3> {
    fn from(value: ValueVector3) -> Self {
        let base_sampler = Sampler::<Vec3>::from(value.clone());

        let mut prob_curves = Vec::new();

        if let Some(dynamics) = value.dynamics.as_ref() {
            if let Some(tables) = dynamics.probability_tables.as_ref() {
                for table_data in tables.iter().cloned() {
                    prob_curves.push(ProbabilityCurve::new(table_data));
                }
            }
        }

        Self {
            base_sampler,
            prob_curves,
        }
    }
}

impl From<ValueVector2> for StochasticSampler<Vec2> {
    fn from(value: ValueVector2) -> Self {
        let base_sampler = Sampler::<Vec2>::from(value.clone());

        let mut prob_curves = Vec::new();

        if let Some(dynamics) = value.dynamics.as_ref() {
            if let Some(tables) = dynamics.probability_tables.as_ref() {
                for table_data in tables.iter().cloned() {
                    prob_curves.push(ProbabilityCurve::new(table_data));
                }
            }
        }

        Self {
            base_sampler,
            prob_curves,
        }
    }
}

impl From<ValueColor> for StochasticSampler<Vec4> {
    fn from(value: ValueColor) -> Self {
        let base_sampler = Sampler::<Vec4>::from(value.clone());

        let mut prob_curves = Vec::new();

        if let Some(dynamics) = value.dynamics.as_ref() {
            if let Some(tables) = dynamics.probability_tables.as_ref() {
                for table_data in tables.iter().cloned() {
                    prob_curves.push(ProbabilityCurve::new(table_data));
                }
            }
        }

        Self {
            base_sampler,
            prob_curves,
        }
    }
}

pub trait FromVfxOption<V, T> {
    fn from_option(option: Option<V>, default_constant: T) -> Self;
}

impl FromVfxOption<ValueFloat, f32> for StochasticSampler<f32> {
    fn from_option(option: Option<ValueFloat>, default_constant: f32) -> Self {
        option
            .unwrap_or(ValueFloat {
                dynamics: None,
                constant_value: Some(default_constant),
            })
            .into()
    }
}

impl FromVfxOption<ValueVector3, Vec3> for StochasticSampler<Vec3> {
    fn from_option(option: Option<ValueVector3>, default_constant: Vec3) -> Self {
        option
            .unwrap_or(ValueVector3 {
                dynamics: None,
                constant_value: Some(default_constant),
            })
            .into()
    }
}

impl FromVfxOption<ValueVector2, Vec2> for StochasticSampler<Vec2> {
    fn from_option(option: Option<ValueVector2>, default_constant: Vec2) -> Self {
        option
            .unwrap_or(ValueVector2 {
                dynamics: None,
                constant_value: Some(default_constant),
            })
            .into()
    }
}

impl FromVfxOption<ValueColor, Vec4> for StochasticSampler<Vec4> {
    fn from_option(option: Option<ValueColor>, default_constant: Vec4) -> Self {
        option
            .unwrap_or(ValueColor {
                dynamics: None,
                constant_value: Some(default_constant),
            })
            .into()
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

pub trait CombineMultiplicative {
    fn combine_mul(&self, components: &[f32]) -> Self;
}

impl CombineMultiplicative for f32 {
    fn combine_mul(&self, components: &[f32]) -> Self {
        let rand_multiplier = components.get(0).unwrap_or(&1.0);
        self * rand_multiplier
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
