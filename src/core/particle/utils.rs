use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use league_core::ValueFloat;

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

pub enum Sampler {
    Constant(f32),
    Curve(UnevenSampleAutoCurve<f32>),
}

impl Curve<f32> for Sampler {
    fn sample_clamped(&self, t: f32) -> f32 {
        match self {
            Self::Constant(v) => *v,
            Self::Curve(c) => c.sample_clamped(t),
        }
    }

    fn domain(&self) -> Interval {
        match self {
            Self::Constant(v) => Interval::new(*v, *v).unwrap(),
            Self::Curve(c) => c.domain(),
        }
    }

    fn sample_unchecked(&self, t: f32) -> f32 {
        match self {
            Self::Constant(v) => *v,
            Self::Curve(c) => c.sample_unchecked(t),
        }
    }
}

impl From<ValueFloat> for Sampler {
    fn from(value: ValueFloat) -> Self {
        if let Some(dynamics) = value.dynamics {
            Self::Curve(
                UnevenSampleAutoCurve::new(dynamics.times.into_iter().zip(dynamics.values))
                    .unwrap(),
            )
        } else {
            Self::Constant(value.constant_value.unwrap())
        }
    }
}
