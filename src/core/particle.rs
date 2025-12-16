mod emitter;
mod environment;
mod particle;
mod skinned_mesh;
mod utils;

use bevy::mesh::{MeshVertexAttribute, VertexFormat};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::transform::systems::{
    mark_dirty_trees, propagate_parent_transforms, sync_simple_transforms,
};
pub use emitter::*;
pub use environment::*;
use league_core::{
    ValueColor, ValueFloat, ValueVector2, ValueVector3, VfxEmitterDefinitionData,
    VfxSystemDefinitionData,
};
use league_utils::hash_wad;
use lol_config::{HashKey, LoadHashKeyTrait};
pub use particle::*;
pub use skinned_mesh::*;
pub use utils::*;

use crate::{Lifetime, LifetimeMode};

pub const ATTRIBUTE_WORLD_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertext_World_Position", 7, VertexFormat::Float32x3);

pub const ATTRIBUTE_UV_FRAME: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertext_UV_FRAME", 8, VertexFormat::Float32x4);

pub const ATTRIBUTE_LIFETIME: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertext_LIFETIME", 9, VertexFormat::Float32x2);

pub const ATTRIBUTE_UV_MULT: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertext_UV_MULT", 99, VertexFormat::Float32x2);

#[derive(Default)]
pub struct PluginParticle;

impl Plugin for PluginParticle {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_particle_spawn);
        app.add_observer(on_command_particle_despawn);

        app.add_plugins(MaterialPlugin::<ParticleMaterialQuad>::default());
        app.add_plugins(MaterialPlugin::<ParticleMaterialQuadSlice>::default());
        app.add_plugins(MaterialPlugin::<ParticleMaterialUnlitDecal>::default());
        app.add_plugins(MaterialPlugin::<ParticleMaterialMesh>::default());
        app.add_plugins(MaterialPlugin::<ParticleMaterialSkinnedMeshParticle>::default());

        app.init_asset::<ParticleMaterialQuad>();
        app.init_asset::<ParticleMaterialQuadSlice>();
        app.init_asset::<ParticleMaterialUnlitDecal>();
        app.init_asset::<ParticleMaterialMesh>();
        app.init_asset::<ParticleMaterialSkinnedMeshParticle>();

        app.init_resource::<ParticleMesh>();

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
                update_emitter,
                update_emitter_attached,
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

    let vfx_system_definition_data = res_assets_vfx_system_definition_data
        .load_hash(trigger.hash)
        .unwrap();

    // if !vfx_system_definition_data
    //     .particle_name
    //     .ends_with("Dash_Trail_ground")
    // {
    //     return;
    // }

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

    for (i, vfx_emitter_definition_data) in vfx_emitter_definition_datas.into_iter().enumerate() {
        // if vfx_emitter_definition_data.emitter_name.clone().unwrap() != "Fiora_Flash" {
        //     continue;
        // }

        let rate = vfx_emitter_definition_data.rate.clone().unwrap();
        let particle_lifetime = vfx_emitter_definition_data
            .particle_lifetime
            .clone()
            .unwrap_or(ValueFloat {
                dynamics: None,
                constant_value: Some(1.0),
            });
        let color = vfx_emitter_definition_data
            .color
            .clone()
            .unwrap_or(ValueColor {
                dynamics: None,
                constant_value: Some(Vec4::ONE),
            });
        let scale0 = vfx_emitter_definition_data
            .scale0
            .clone()
            .unwrap_or(ValueVector3 {
                dynamics: None,
                constant_value: Some(Vec3::ONE),
            });
        let birth_velocity = vfx_emitter_definition_data
            .birth_velocity
            .clone()
            .unwrap_or(ValueVector3 {
                dynamics: None,
                constant_value: Some(Vec3::ZERO),
            });
        let birth_acceleration = vfx_emitter_definition_data
            .birth_acceleration
            .clone()
            .unwrap_or(ValueVector3 {
                dynamics: None,
                constant_value: Some(Vec3::ZERO),
            });
        let birth_color = vfx_emitter_definition_data
            .birth_color
            .clone()
            .unwrap_or(ValueColor {
                dynamics: None,
                constant_value: Some(Vec4::ONE),
            });
        let birth_rotation0 = vfx_emitter_definition_data
            .birth_rotation0
            .clone()
            .unwrap_or(ValueVector3 {
                dynamics: None,
                constant_value: Some(Vec3::ZERO),
            });
        let birth_scale0 =
            vfx_emitter_definition_data
                .birth_scale0
                .clone()
                .unwrap_or(ValueVector3 {
                    dynamics: None,
                    constant_value: Some(Vec3::ONE),
                });
        let birth_uv_offset = vfx_emitter_definition_data
            .birth_uv_offset
            .clone()
            .unwrap_or(ValueVector2 {
                dynamics: None,
                constant_value: Some(Vec2::ONE),
            });
        let birth_uv_scroll_rate = vfx_emitter_definition_data
            .birth_uv_scroll_rate
            .clone()
            .unwrap_or(ValueVector2 {
                dynamics: None,
                constant_value: Some(Vec2::ZERO),
            });
        let emitter_position = vfx_emitter_definition_data
            .emitter_position
            .clone()
            .unwrap_or(ValueVector3 {
                dynamics: None,
                constant_value: Some(Vec3::ZERO),
            });
        let bind_weight = vfx_emitter_definition_data
            .bind_weight
            .clone()
            .unwrap_or(ValueFloat {
                dynamics: None,
                constant_value: Some(0.0),
            });

        commands.entity(entity).with_related::<EmitterOf>((
            ParticleId {
                hash: trigger.hash,
                index: i,
            },
            ParticleEmitterState {
                birth_acceleration: birth_acceleration.into(),
                birth_color: birth_color.into(),
                birth_rotation0: birth_rotation0.into(),
                birth_scale0: birth_scale0.into(),
                birth_uv_offset: birth_uv_offset.into(),
                birth_uv_scroll_rate: birth_uv_scroll_rate.into(),
                birth_velocity: birth_velocity.into(),
                bind_weight: bind_weight.into(),
                color: color.into(),
                scale0: scale0.into(),
                emission_debt: 0.,
                particle_lifetime: particle_lifetime.into(),
                rate: rate.into(),
                emitter_position: emitter_position.into(),
                global_transform,
            },
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
