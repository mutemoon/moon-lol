use bevy::prelude::*;
use lol_base_render::particle::{ConfigVfxEmitterDefinition, StochasticSampler};

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
    /// 发射器世界朝向覆盖（来自 CommandParticleSpawn.rotation）；
    /// Some 时 update_emitter_position 用它替换锚点朝向，使定向粒子
    /// 始终朝向指定方向
    pub rotation_override: Option<Quat>,
    pub is_direction_oriented: bool,
    /// 发射器级别共享材质句柄（Quad/Distortion/Decal 粒子共享复用）
    pub cached_material: Option<Handle<crate::particle::dynamic::ParticleMaterialDynamic>>,
}

impl ParticleEmitterState {
    pub fn new(
        def: &ConfigVfxEmitterDefinition,
        global_transform: Transform,
        rotation_override: Option<Quat>,
    ) -> Self {
        Self {
            birth_acceleration: def.birth_acceleration.clone(),
            birth_color: def.birth_color.clone(),
            birth_rotation0: def.birth_rotation0.clone(),
            birth_scale0: def.birth_scale0.clone(),
            birth_uv_offset: def.birth_uv_offset.clone(),
            birth_uv_scroll_rate: def.birth_uv_scroll_rate.clone(),
            birth_velocity: def.birth_velocity.clone(),
            bind_weight: def.bind_weight.clone(),
            color: def.color.clone(),
            scale0: def.scale0.clone(),
            emission_debt: 0.,
            particle_lifetime: def.particle_lifetime.clone(),
            rate: def.rate.clone(),
            emitter_position: def.emitter_position.clone(),
            global_transform,
            rotation_override,
            is_direction_oriented: def.is_direction_oriented.unwrap_or(false),
            cached_material: None,
        }
    }
}

#[derive(Component, Debug)]
#[relationship(relationship_target = Emitters)]
pub struct EmitterOf(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = EmitterOf, linked_spawn)]
pub struct Emitters(Vec<Entity>);
