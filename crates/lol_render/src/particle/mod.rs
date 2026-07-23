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
use league_utils::hash_wad;
use lol_base::hash_key::{HashKey, LoadHashKeyTrait};
use lol_base::particle::{
    ConfigResourceResolver, ConfigVfx, ConfigVfxEmitterDefinition, ConfigVfxSystemDefinition,
};
use lol_core::lifetime::{Lifetime, LifetimeMode};

use crate::loaders::shader::{LeagueLoaderShaderMap, ShaderMapAsset};
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
use crate::particle::utils::ResourceCache;
use crate::shader::{
    DebugShaderHandles, ResourceShaderMapHandle, ShaderCheckTimer, debug_check_shaders,
    startup_load_shaders, update_shaders,
};

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

/// 持有 ConfigVfx 的 Handle，防止资产被卸载
#[derive(Component)]
pub struct VfxHandle(pub Handle<ConfigVfx>);

#[derive(Default)]
pub struct PluginParticle;

impl Plugin for PluginParticle {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_particle_spawn);
        app.add_observer(on_command_particle_despawn);

        app.init_asset_loader::<LeagueLoaderShaderMap>();
        app.init_asset_loader::<crate::loaders::vfx::LoaderConfigVfxLoader>();
        app.init_asset_loader::<crate::loaders::scb::ScbMeshLoader>();

        app.init_asset::<ShaderMapAsset>();
        app.init_asset::<ConfigVfx>();
        app.init_asset::<ConfigVfxSystemDefinition>();
        app.init_asset::<ConfigResourceResolver>();

        app.add_systems(PostUpdate, inject_vfx_assets);

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
        app.init_resource::<ResourceShaderMapHandle>();
        app.init_resource::<DebugShaderHandles>();
        app.init_resource::<ShaderCheckTimer>();
        app.init_resource::<ResourceCache>();

        app.add_systems(Startup, startup_load_shaders);
        app.add_systems(Update, (update_shaders, debug_check_shaders));

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
    pub vfx_handle: HashKey<ConfigVfxSystemDefinition>,
    pub index: usize,
}

impl ParticleId {
    pub fn get_def<'a>(
        self: &Self,
        res_assets_vfx_system_definition_data: &'a Res<Assets<ConfigVfxSystemDefinition>>,
    ) -> &'a ConfigVfxEmitterDefinition {
        let system_def = res_assets_vfx_system_definition_data
            .load_hash(self.vfx_handle)
            .unwrap();

        let complex_len = system_def
            .complex_emitter_definition_data
            .as_ref()
            .map_or(0, |v| v.len());

        if self.index < complex_len {
            &system_def.complex_emitter_definition_data.as_ref().unwrap()[self.index]
        } else {
            let simple_idx = self.index - complex_len;
            &system_def.simple_emitter_definition_data.as_ref().unwrap()[simple_idx]
        }
    }
}

#[derive(EntityEvent)]
pub struct CommandParticleSpawn {
    pub entity: Entity,
    /// 指向角色的 vfx.ron 文件对应的 ConfigVfx 资产
    pub vfx_handle: HashKey<ConfigVfxSystemDefinition>,
}

#[derive(EntityEvent)]
pub struct CommandParticleDespawn {
    pub entity: Entity,
    pub vfx_handle: HashKey<ConfigVfxSystemDefinition>,
}

impl ParticleMesh {
    pub fn get_mesh_handle(self: &Self, path: &str) -> Option<Handle<Mesh>> {
        return self.0.get(&hash_wad(path)).cloned();
    }
}

fn on_command_particle_spawn(
    trigger: On<CommandParticleSpawn>,
    mut commands: Commands,
    res_assets_vfx_system_definition_data: Res<Assets<ConfigVfxSystemDefinition>>,
    q_global_transform: Query<&GlobalTransform>,
) {
    let entity = trigger.event_target();
    info!(
        "{entity} 系统粒子创建命令，vfx_handle={:08x}",
        trigger.vfx_handle.0
    );

    let Ok(global_transform) = q_global_transform
        .get(entity)
        .map(|v| v.compute_transform())
    else {
        info!("{entity} 找不到 GlobalTransform，跳过粒子创建");
        return;
    };

    let Some(vfx_system_def) = res_assets_vfx_system_definition_data.load_hash(trigger.vfx_handle)
    else {
        info!(
            "{entity} 找不到 ConfigVfxSystemDefinition(vfx_handle={:08x})，跳过粒子创建",
            trigger.vfx_handle.0
        );
        return;
    };
    info!(
        "{entity} VFX 系统定义已加载，粒子名称={:?}，路径={:?}",
        vfx_system_def.particle_name, vfx_system_def.particle_path
    );

    let complex_count = vfx_system_def
        .complex_emitter_definition_data
        .as_ref()
        .map_or(0, |v| v.len());
    let simple_count = vfx_system_def
        .simple_emitter_definition_data
        .as_ref()
        .map_or(0, |v| v.len());
    info!("{entity} 发射器数量：complex={complex_count} simple={simple_count}");

    let vfx_emitter_definition_datas = vfx_system_def
        .complex_emitter_definition_data
        .iter()
        .flatten()
        .chain(
            vfx_system_def
                .simple_emitter_definition_data
                .iter()
                .flatten(),
        );

    for (i, vfx_emitter_definition_data) in vfx_emitter_definition_datas.enumerate().take(1) {
        let emitter_name = vfx_emitter_definition_data
            .emitter_name
            .as_deref()
            .unwrap_or("(无名称)");
        info!(
            "{entity} 创建发射器[{i}] name={emitter_name:?} lifetime={:?} is_single_particle={:?}",
            vfx_emitter_definition_data.lifetime, vfx_emitter_definition_data.is_single_particle,
        );
        commands.entity(entity).with_related::<EmitterOf>((
            ParticleId {
                vfx_handle: trigger.vfx_handle,
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
    info!(
        "{entity} 粒子创建完成，共创建 {} 个发射器",
        complex_count + simple_count
    );
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

        if particle.vfx_handle == trigger.vfx_handle {
            commands.entity(emitter).despawn();
        }
    }
}

/// 当 ConfigVfx 加载完成后，将其中所有 system/resolver 注入到对应的 Assets 中，
/// 使 ParticleId::get_def 能通过 system_hash 直接查到定义。
fn inject_vfx_assets(
    res_assets_vfx: Res<Assets<ConfigVfx>>,
    mut asset_events: MessageReader<AssetEvent<ConfigVfx>>,
    mut res_assets_vfx_system: ResMut<Assets<ConfigVfxSystemDefinition>>,
    mut res_assets_vfx_resolver: ResMut<Assets<ConfigResourceResolver>>,
) {
    for event in asset_events.read() {
        let id = match event {
            AssetEvent::Added { id } => {
                info!("检测到 ConfigVfx 加载完成，id={id:?}");
                id
            }
            AssetEvent::Modified { id } => {
                info!("ConfigVfx 已修改，重新注入，id={id:?}");
                id
            }
            _ => {
                info!("ConfigVfx 事件类型不匹配，跳过{:?}", event);
                continue;
            }
        };
        let Some(config_vfx) = res_assets_vfx.get(*id) else {
            info!("ConfigVfx(id={id:?}) 已存在事件但无法获取资产内容");
            continue;
        };
        info!(
            "注入 {} 个 system 和 {} 个 resolver",
            config_vfx.systems.len(),
            config_vfx.resolvers.len(),
        );
        for (&hash, system_def) in &config_vfx.systems {
            info!(
                "  注入 system hash={:08x} name={:?}",
                hash, system_def.particle_name
            );
            res_assets_vfx_system.add_hash(hash, system_def.clone());
        }
        for (&hash, resolver) in &config_vfx.resolvers {
            info!(
                "  注入 resolver hash={:08x} 条目数={}",
                hash,
                resolver.resource_map.len(),
            );
            res_assets_vfx_resolver.add_hash(hash, resolver.clone());
        }
    }
}
