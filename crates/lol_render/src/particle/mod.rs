pub mod emitters;
pub mod environment;
pub mod particle;
pub mod skinned_mesh;
pub mod utils;

use bevy::mesh::{MeshVertexAttribute, VertexFormat};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::transform::systems::{
    mark_dirty_trees, propagate_parent_transforms, sync_simple_transforms,
};
use league_core::extract::{VfxEmitterDefinitionData, VfxSystemDefinitionData};
use league_utils::hash_wad;
use lol_base::prop::{HashKey, LoadHashKeyTrait};
use lol_core::lifetime::{Lifetime, LifetimeMode};

// use crate::loaders::shader::LeagueLoaderShaderToc;
use crate::particle::emitters::decal::update_decal_intersections;
use crate::particle::emitters::distortion::update_emitter_distortion;
use crate::particle::emitters::mesh::update_emitter_mesh;
use crate::particle::emitters::position::update_emitter_position;
use crate::particle::emitters::quad::update_emitter_quad;
use crate::particle::emitters::skinned_mesh::update_emitter_skinned_mesh;
use crate::particle::emitters::state::{EmitterOf, Emitters, ParticleEmitterState};
use crate::particle::emitters::unlit_decal::update_emitter_decal;
use crate::particle::environment::unlit_decal::ParticleMaterialUnlitDecal;
use crate::particle::particle::distortion::ParticleMaterialDistortion;
use crate::particle::particle::mesh::ParticleMaterialMesh;
use crate::particle::particle::quad::ParticleMaterialQuad;
use crate::particle::particle::quad_slice::ParticleMaterialQuadSlice;
use crate::particle::particle::{
    update_particle, update_particle_skinned_mesh_particle, update_particle_transform,
};
use crate::particle::skinned_mesh::particle::ParticleMaterialSkinnedMeshParticle;
use crate::shader::{ResourceShaderHandles, startup_load_shaders, update_shaders};

pub const ATTRIBUTE_WORLD_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("ATTRIBUTE_WORLD_POSITION", 2020, VertexFormat::Float32x3);

pub const ATTRIBUTE_WORLD_POSITION_VEC4: MeshVertexAttribute = MeshVertexAttribute::new(
    "ATTRIBUTE_WORLD_POSITION_VEC4",
    2021,
    VertexFormat::Float32x4,
);

pub const ATTRIBUTE_UV_FRAME: MeshVertexAttribute =
    MeshVertexAttribute::new("ATTRIBUTE_UV_FRAME", 2022, VertexFormat::Float32x4);

pub const ATTRIBUTE_LIFETIME: MeshVertexAttribute =
    MeshVertexAttribute::new("ATTRIBUTE_LIFETIME", 2023, VertexFormat::Float32x2);

pub const ATTRIBUTE_UV_MULT: MeshVertexAttribute =
    MeshVertexAttribute::new("ATTRIBUTE_UV_MULT", 2024, VertexFormat::Float32x2);

#[derive(Default)]
pub struct PluginParticle;

impl Plugin for PluginParticle {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_particle_spawn);
        app.add_observer(on_command_particle_despawn);

        // app.init_asset_loader::<LeagueLoaderShaderToc>();

        app.add_plugins(MaterialPlugin::<ParticleMaterialDistortion>::default());
        app.add_plugins(MaterialPlugin::<ParticleMaterialQuad>::default());
        app.add_plugins(MaterialPlugin::<ParticleMaterialQuadSlice>::default());
        app.add_plugins(MaterialPlugin::<ParticleMaterialUnlitDecal>::default());
        app.add_plugins(MaterialPlugin::<ParticleMaterialMesh>::default());
        app.add_plugins(MaterialPlugin::<ParticleMaterialSkinnedMeshParticle>::default());

        app.init_asset::<ParticleMaterialDistortion>();
        app.init_asset::<ParticleMaterialQuad>();
        app.init_asset::<ParticleMaterialQuadSlice>();
        app.init_asset::<ParticleMaterialUnlitDecal>();
        app.init_asset::<ParticleMaterialMesh>();
        app.init_asset::<ParticleMaterialSkinnedMeshParticle>();

        app.init_resource::<ParticleMesh>();
        app.init_resource::<ResourceShaderHandles>();

        app.add_systems(Startup, startup_load_shaders);
        app.add_systems(Update, update_shaders);

        app.add_systems(
            PostUpdate,
            (
                update_emitter_position,
                (
                    mark_dirty_trees,
                    propagate_parent_transforms,
                    sync_simple_transforms,
                )
                    .chain(),
                update_emitter_quad,
                update_emitter_decal,
                update_emitter_mesh,
                update_emitter_skinned_mesh,
                update_emitter_distortion,
                update_decal_intersections,
                update_particle_transform,
                (
                    mark_dirty_trees,
                    propagate_parent_transforms,
                    sync_simple_transforms,
                )
                    .chain(),
                update_particle,
                update_particle_skinned_mesh_particle,
            )
                .chain()
                .after(TransformSystems::Propagate),
        );
    }
}

#[derive(Resource, Default)]
pub struct ParticleMesh(HashMap<u64, Handle<Mesh>>);

#[derive(Component, Clone, Debug)]
pub struct ParticleId {
    hash: HashKey<VfxSystemDefinitionData>,
    index: usize,
}

impl ParticleId {
    pub fn get_def<'a>(
        self: &Self,
        res_assets_vfx_system_definition_data: &'a Res<Assets<VfxSystemDefinitionData>>,
    ) -> &'a VfxEmitterDefinitionData {
        res_assets_vfx_system_definition_data
            .load_hash(self.hash)
            .unwrap()
            .complex_emitter_definition_data
            .as_ref()
            .unwrap()
            .get(self.index)
            .unwrap()
    }
}

#[derive(EntityEvent)]
pub struct CommandParticleSpawn {
    pub entity: Entity,
    pub hash: HashKey<VfxSystemDefinitionData>,
}

#[derive(EntityEvent)]
pub struct CommandParticleDespawn {
    pub entity: Entity,
    pub hash: HashKey<VfxSystemDefinitionData>,
}

impl ParticleMesh {
    pub fn get_mesh_handle(self: &Self, path: &str) -> Option<Handle<Mesh>> {
        return self.0.get(&hash_wad(path)).cloned();
    }
}

fn on_command_particle_spawn(
    trigger: On<CommandParticleSpawn>,
    mut commands: Commands,
    res_assets_vfx_system_definition_data: Res<Assets<VfxSystemDefinitionData>>,
    q_global_transform: Query<&GlobalTransform>,
) {
    let entity = trigger.event_target();

    let Ok(global_transform) = q_global_transform
        .get(entity)
        .map(|v| v.compute_transform())
    else {
        return;
    };

    let Some(vfx_system_definition_data) =
        res_assets_vfx_system_definition_data.load_hash(trigger.hash)
    else {
        return;
    };

    let vfx_emitter_definition_datas = vfx_system_definition_data
        .complex_emitter_definition_data
        .iter()
        .flatten()
        .chain(
            vfx_system_definition_data
                .simple_emitter_definition_data
                .iter()
                .flatten(),
        );

    for (i, vfx_emitter_definition_data) in vfx_emitter_definition_datas.enumerate()
    // .skip(2).take(1)
    {
        println!("{:?}", vfx_emitter_definition_data.emitter_name);
        commands.entity(entity).with_related::<EmitterOf>((
            ParticleId {
                hash: trigger.hash,
                index: i,
            },
            ParticleEmitterState::new(vfx_emitter_definition_data, global_transform),
            Lifetime::new(
                vfx_emitter_definition_data.lifetime.unwrap_or(1.0),
                LifetimeMode::TimerAndNoChildren,
            ),
            global_transform,
        ));
    }
}

fn on_command_particle_despawn(
    trigger: On<CommandParticleDespawn>,
    mut commands: Commands,
    q_emitters: Query<&Emitters>,
    q_emitter: Query<&ParticleId>,
) {
    let Ok(emitters) = q_emitters.get(trigger.event_target()) else {
        return;
    };

    for emitter in emitters.iter() {
        let Ok(particle) = q_emitter.get(emitter) else {
            continue;
        };

        if particle.hash == trigger.hash {
            commands.entity(emitter).despawn();
        }
    }
}
