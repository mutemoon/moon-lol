use bevy::prelude::*;

use lol_config::ConfigMap;

use crate::particles::{
    update_emitter, update_particle, ParticleEmitterState, QuadMaterial, QuadSliceMaterial,
};

#[derive(Default)]
pub struct PluginParticle;

impl Plugin for PluginParticle {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_particle_spawn);

        app.add_plugins(MaterialPlugin::<QuadMaterial>::default());
        app.add_plugins(MaterialPlugin::<QuadSliceMaterial>::default());

        app.init_asset::<QuadMaterial>();
        app.init_asset::<QuadSliceMaterial>();

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
