mod emitter;
mod particle;
mod ps;
mod utils;
mod vs;

pub use emitter::*;
pub use particle::*;
pub use ps::*;
pub use utils::*;
pub use vs::*;

use bevy::prelude::*;
use bevy::render::mesh::{MeshVertexAttribute, VertexFormat};

use league_core::ValueFloat;
use lol_config::ConfigMap;

pub const ATTRIBUTE_WORLD_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertext_World_Position", 7, VertexFormat::Float32x3);

pub const ATTRIBUTE_UV_FRAME: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertext_Frame", 8, VertexFormat::Float32x4);

pub const ATTRIBUTE_LIFETIME: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertext_Life", 9, VertexFormat::Float32x2);

#[derive(Default)]
pub struct PluginParticle;

impl Plugin for PluginParticle {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_particle_spawn);
        app.add_observer(on_command_particle_despawn);

        app.add_plugins(MaterialPlugin::<QuadMaterial>::default());
        app.add_plugins(MaterialPlugin::<QuadSliceMaterial>::default());

        app.init_asset::<QuadMaterial>();
        app.init_asset::<QuadSliceMaterial>();

        app.add_systems(Update, update_emitter);
        app.add_systems(Last, update_particle);
    }
}

#[derive(Component, Clone)]
pub struct Particle(pub u32);

#[derive(Event)]
pub struct CommandParticleSpawn {
    pub particle: u32,
}

#[derive(Event)]
pub struct CommandParticleDespawn {
    pub particle: u32,
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

    for vfx_emitter_definition_data in vfx_emitter_definition_datas.into_iter() {
        let birth_rotation = vfx_emitter_definition_data.birth_rotation0.clone().unwrap();

        commands.entity(trigger.target()).with_child((
            vfx_emitter_definition_data.clone(),
            Particle(trigger.particle),
            ParticleEmitterState {
                timer: Timer::from_seconds(
                    vfx_emitter_definition_data.lifetime.unwrap_or(10.0),
                    TimerMode::Repeating,
                ),
                rate_sampler: vfx_emitter_definition_data.rate.clone().unwrap().into(),
                life_sampler: vfx_emitter_definition_data
                    .particle_lifetime
                    .clone()
                    .unwrap_or(ValueFloat {
                        dynamics: None,
                        constant_value: Some(1.0),
                    })
                    .into(),
                rotation_sampler: birth_rotation.into(),
                emission_debt: 1.0,
            },
            Transform::default(),
        ));
    }
}

fn on_command_particle_despawn(
    trigger: Trigger<CommandParticleDespawn>,
    mut commands: Commands,
    q_children: Query<&Children>,
    q_particle_emitter: Query<(Entity, &Particle)>,
) {
    let Ok(children) = q_children.get(trigger.target()) else {
        return;
    };

    for child in children.iter() {
        let Ok((emitter_or_particle_entity, particle)) = q_particle_emitter.get(child) else {
            continue;
        };

        if particle.0 == trigger.particle {
            commands.entity(emitter_or_particle_entity).despawn();
        }
    }
}
