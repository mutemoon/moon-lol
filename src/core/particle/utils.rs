use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use league_core::{ValueFloat, ValueVector3};

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
                    Self::Curve(
                        UnevenSampleAutoCurve::new(dynamics.times.into_iter().zip(dynamics.values))
                            .unwrap(),
                    )
                } else {
                    Self::Constant(value.constant_value.unwrap())
                }
            }
        }
    };
}

impl_sampler_traits!(Sampler<f32>, ValueFloat, f32, Interval::EVERYWHERE);

impl_sampler_traits!(Sampler<Vec3>, ValueVector3, Vec3, Interval::EVERYWHERE);
