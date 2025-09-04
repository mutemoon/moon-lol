use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::{
            MeshVertexAttribute, MeshVertexBufferLayoutRef, VertexAttributeValues, VertexFormat,
        },
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, ShaderType,
            SpecializedMeshPipelineError,
        },
    },
};

use league_core::{ValueFloat, VfxEmitterDefinitionData, VfxEmitterDefinitionDataPrimitive};
use lol_config::ConfigMap;

use crate::particles::ParticleQuad;

#[derive(Default)]
pub struct PluginParticle;

impl Plugin for PluginParticle {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_particle_spawn);

        app.add_plugins(MaterialPlugin::<CustomMaterial>::default());

        app.init_asset::<CustomMaterial>();

        app.add_systems(Update, update_emitter);
        app.add_systems(First, update_particle);
    }
}

#[derive(Component)]
pub struct Particle(pub u32);

#[derive(Event)]
pub struct CommandParticleSpawn {
    pub particle: u32,
}

#[derive(Clone, ShaderType)]
pub struct UniformsVertex {
    pub fog_of_war_params: Vec4,
    pub fog_of_war_always_below_y: Vec4,
    pub fow_height_fade: Vec4,
    pub particle_depth_push_pull: f32,
    pub texture_info: Vec4,
}

impl Default for UniformsVertex {
    fn default() -> Self {
        Self {
            fog_of_war_params: Vec4::ZERO,
            fog_of_war_always_below_y: Vec4::ZERO,
            fow_height_fade: Vec4::ZERO,
            particle_depth_push_pull: 0.0,
            texture_info: Vec4::ONE,
        }
    }
}

pub const ATTRIBUTE_WORLD_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertext_World_Position", 7, VertexFormat::Float32x3);

pub const ATTRIBUTE_UV_FRAME: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertext_Frame", 8, VertexFormat::Float32x4);

pub const ATTRIBUTE_LIFETIME: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertext_Life", 9, VertexFormat::Float32x2);

#[derive(Clone, ShaderType)]
pub struct UniformsPixel {
    pub slice_range: Vec2,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct CustomMaterial {
    #[uniform(0)]
    pub uniforms_vertex: UniformsVertex,
    #[uniform(1)]
    pub uniforms_pixel: UniformsPixel,
    #[texture(2)]
    #[sampler(3)]
    pub particle_color_texture: Option<Handle<Image>>,
    #[texture(4)]
    #[sampler(5)]
    pub texture: Option<Handle<Image>>,
    #[texture(6)]
    #[sampler(7)]
    pub cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: Option<Handle<Image>>,
    #[texture(8)]
    #[sampler(9)]
    pub sampler_fow: Option<Handle<Image>>,
    pub is_local_orientation: bool,
    pub alpha_mode: AlphaMode,
}

impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ps_quad_ps_slice/BASE.frag".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/vs_quad_vs/BASE.vert".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    // Bevy assumes by default that vertex shaders use the "vertex" entry point
    // and fragment shaders use the "fragment" entry point (for WGSL shaders).
    // GLSL uses "main" as the entry point, so we must override the defaults here
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();

        let vertex_layout = layout.0.get_layout(&[
            ATTRIBUTE_WORLD_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_COLOR.at_shader_location(3),
            ATTRIBUTE_UV_FRAME.at_shader_location(8),
            ATTRIBUTE_LIFETIME.at_shader_location(9),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}

#[derive(Component)]
pub struct ParticleEmitterState {
    pub timer: Timer,
    pub rate_sampler: Sampler,
    pub life_sampler: Sampler,
    pub emission_debt: f32,
}

#[derive(Component)]
pub struct ParticleLifeState {
    pub timer: Timer,
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

fn on_command_particle_spawn(
    trigger: Trigger<CommandParticleSpawn>,
    mut commands: Commands,
    res_config_map: Res<ConfigMap>,
) {
    let vfx_system_definition_data = res_config_map
        .vfx_system_definition_datas
        .get(&trigger.particle)
        .unwrap();

    let mut vfx_emitter_definition_datas = Vec::new();

    if let Some(complex_emitter_definition_data) =
        &vfx_system_definition_data.complex_emitter_definition_data
    {
        vfx_emitter_definition_datas.extend(complex_emitter_definition_data);
    }

    if let Some(simple_emitter_definition_data) =
        &vfx_system_definition_data.simple_emitter_definition_data
    {
        vfx_emitter_definition_datas.extend(simple_emitter_definition_data);
    }

    for vfx_emitter_definition_data in vfx_emitter_definition_datas.into_iter().take(1) {
        commands.entity(trigger.target()).with_child((
            vfx_emitter_definition_data.clone(),
            ParticleEmitterState {
                timer: Timer::from_seconds(
                    vfx_emitter_definition_data.lifetime.unwrap_or(1.0),
                    TimerMode::Repeating,
                ),
                rate_sampler: vfx_emitter_definition_data.rate.clone().unwrap().into(),
                life_sampler: vfx_emitter_definition_data
                    .particle_lifetime
                    .clone()
                    .unwrap()
                    .into(),
                emission_debt: 1.0,
            },
            Transform::default(),
        ));
    }
}

fn update_emitter(
    mut commands: Commands,
    mut res_mesh: ResMut<Assets<Mesh>>,
    res_asset_server: Res<AssetServer>,
    mut res_material: ResMut<Assets<CustomMaterial>>,
    mut query: Query<(
        Entity,
        &ChildOf,
        &mut ParticleEmitterState,
        &VfxEmitterDefinitionData,
    )>,
    time: Res<Time>,
) {
    for (entity, parent, mut emitter, vfx_emitter_definition_data) in query.iter_mut() {
        emitter.timer.tick(time.delta());

        if emitter.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let normalized_time = emitter.timer.elapsed_secs() / emitter.timer.duration().as_secs_f32();

        let current_rate = emitter.rate_sampler.sample_clamped(normalized_time);

        // 计算本帧应该发射的粒子数量
        // 加上一帧的 "欠账"，这使得在低速率下也能平滑发射
        let particles_to_spawn_f32 = current_rate * time.delta_secs() + emitter.emission_debt;

        // 向下取整，得到本帧实际生成的整数粒子数
        let particles_to_spawn = particles_to_spawn_f32.floor() as u32;

        // 更新 "欠账"，为下一帧做准备
        emitter.emission_debt = particles_to_spawn_f32.fract();

        let lifetime = emitter.life_sampler.sample_clamped(normalized_time);

        for _ in 0..particles_to_spawn {
            let mesh = vfx_emitter_definition_data.primitive.clone().unwrap();

            let mesh = match mesh {
                VfxEmitterDefinitionDataPrimitive::VfxPrimitiveArbitraryQuad => {
                    ParticleQuad::default()
                }
                _ => todo!(),
            };

            let mesh = res_mesh.add(mesh);

            let texture = vfx_emitter_definition_data
                .texture
                .as_ref()
                .map(|v| res_asset_server.load(v))
                .unwrap();

            let particle_color_texture = vfx_emitter_definition_data
                .particle_color_texture
                .as_ref()
                .unwrap();

            let particle_color_texture = res_asset_server.load(particle_color_texture);

            let range = vfx_emitter_definition_data
                .slice_technique_range
                .unwrap_or(0.0);

            let material = res_material.add(CustomMaterial {
                uniforms_pixel: UniformsPixel {
                    slice_range: vec2(range, 1.0 / (range * range)),
                },
                texture: Some(texture),
                particle_color_texture: Some(particle_color_texture),
                alpha_mode: AlphaMode::Blend,
                is_local_orientation: vfx_emitter_definition_data
                    .is_local_orientation
                    .unwrap_or(false),
                cmb_tex_pixel_color_remap_ramp_smp_clamp_no_mip: None,
                uniforms_vertex: UniformsVertex::default(),
                sampler_fow: None,
            });

            let target = commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(vec3(0.0, 100.0, 0.0)),
                ParticleLifeState {
                    timer: Timer::from_seconds(lifetime, TimerMode::Repeating),
                },
            ));

            let target_id = target.id();

            commands.entity(parent.0).add_child(target_id);
        }
    }
}

fn update_particle(
    mut commands: Commands,
    mut res_material: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, &GlobalTransform, &Mesh3d, &mut ParticleLifeState)>,
    time: Res<Time>,
) {
    for (entity, transform, mesh_material, mut particle) in query.iter_mut() {
        let mesh = res_material.get_mut(mesh_material).unwrap();
        particle.timer.tick(time.delta());

        if particle.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let lifetime = particle.timer.elapsed_secs() / particle.timer.duration().as_secs_f32();

        let lifetime_values = mesh.attribute_mut(ATTRIBUTE_LIFETIME).unwrap();

        match lifetime_values {
            VertexAttributeValues::Float32x2(items) => {
                for item in items {
                    item[0] = lifetime;
                }
            }
            _ => panic!(),
        }

        let VertexAttributeValues::Float32x3(postion_values) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap()
        else {
            panic!();
        };

        let transition = transform.translation();

        let postion_values = postion_values
            .iter_mut()
            .map(|v| {
                [
                    v[0] + transition.x,
                    v[1] + transition.y,
                    v[2] + transition.z,
                ]
            })
            .collect::<Vec<_>>();

        mesh.insert_attribute(ATTRIBUTE_WORLD_POSITION, postion_values);
    }
}
