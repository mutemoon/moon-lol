use bevy::prelude::*;
use league_core::extract::VfxEmitterDefinitionData;

use crate::core::particle::utils::{FromVfxOption, StochasticSampler};

#[derive(Component)]
#[require(Visibility)]
pub struct ParticleEmitterState {
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
    pub emission_debt: f32,
    pub particle_lifetime: StochasticSampler<f32>,
    pub rate: StochasticSampler<f32>,
    pub emitter_position: StochasticSampler<Vec3>,
    pub global_transform: Transform,
}

impl ParticleEmitterState {
    pub fn new(def: &VfxEmitterDefinitionData, global_transform: Transform) -> Self {
        Self {
            birth_acceleration: FromVfxOption::from_option(
                def.birth_acceleration.clone(),
                Vec3::ZERO,
            ),
            birth_color: FromVfxOption::from_option(def.birth_color.clone(), Vec4::ONE),
            birth_rotation0: FromVfxOption::from_option(def.birth_rotation0.clone(), Vec3::ZERO),
            birth_scale0: FromVfxOption::from_option(def.birth_scale0.clone(), Vec3::ONE),
            birth_uv_offset: FromVfxOption::from_option(def.birth_uv_offset.clone(), Vec2::ONE),
            birth_uv_scroll_rate: FromVfxOption::from_option(
                def.birth_uv_scroll_rate.clone(),
                Vec2::ZERO,
            ),
            birth_velocity: FromVfxOption::from_option(def.birth_velocity.clone(), Vec3::ZERO),
            bind_weight: FromVfxOption::from_option(def.bind_weight.clone(), 0.0),
            color: FromVfxOption::from_option(def.color.clone(), Vec4::ONE),
            scale0: FromVfxOption::from_option(def.scale0.clone(), Vec3::ONE),
            emission_debt: 0.,
            particle_lifetime: FromVfxOption::from_option(def.particle_lifetime.clone(), 1.0),
            rate: FromVfxOption::from_option(def.rate.clone(), 0.0),
            emitter_position: FromVfxOption::from_option(def.emitter_position.clone(), Vec3::ZERO),
            global_transform,
        }
    }
}

#[derive(Component, Debug)]
#[relationship(relationship_target = Emitters)]
pub struct EmitterOf(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = EmitterOf, linked_spawn)]
pub struct Emitters(Vec<Entity>);
