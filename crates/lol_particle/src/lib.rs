pub mod emitters;
pub mod loaders;
pub mod particle;
pub mod utils;

use bevy::mesh::{MeshVertexAttribute, VertexFormat};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::transform::systems::{
    mark_dirty_trees, propagate_parent_transforms, sync_simple_transforms,
};
use league_utils::{LeagueShader, hash_wad};
use lol_base::hash_key::{HashKey, LoadHashKeyTrait};
use lol_base::render_cmd::CommandSkinSoundPlay;
/// 因为粒子命令事件与 shader 布局描述已下沉到 lol_base_render，
/// 所以在 crate 根部 re-export 保持调用方路径稳定。
pub use lol_base_render::particle::{CommandParticleDespawn, CommandParticleSpawn};
use lol_base_render::particle::{
    ConfigResourceResolver, ConfigVfx, ConfigVfxEmitterDefinition, ConfigVfxHandle,
    ConfigVfxSystemDefinition,
};
use lol_base_render::shader::{DebugShaderHandles, ShaderMap, startup_load_shaders};
pub use lol_base_render::shader_layout;
use lol_core::lifetime::{Lifetime, LifetimeMode, PluginLifetime};

use crate::emitters::decal::update_decal_intersections;
use crate::emitters::position::update_emitter_position;
use crate::emitters::state::{EmitterOf, Emitters, ParticleEmitterState};
use crate::emitters::update::update_emitters;
use crate::particle::dynamic::PluginDynamicMaterial;
use crate::particle::{
    update_particle, update_particle_skinned_mesh_particle, update_particle_transform,
};
use crate::utils::ResourceCache;

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
        // 因为发射器/粒子的年龄推进与到期 despawn 全部依赖 PluginLifetime 的
        // PreUpdate tick，而独立示例（particle_studio 等）不加载 PluginCore，
        // 所以这里兜底注册；完整 App 已由 PluginCore 注册时跳过避免重复
        if !app.is_plugin_added::<PluginLifetime>() {
            app.add_plugins(PluginLifetime);
        }

        app.add_observer(on_command_particle_spawn);
        app.add_observer(on_command_particle_despawn);

        app.init_asset_loader::<crate::loaders::scb::ScbMeshLoader>();

        // ConfigVfx 现为独立 Asset，由 skin{N}.ron 中的 ConfigVfxHandle 资源承载其句柄；
        // 自定义 ConfigVfxLoader 以纯 RON 加载并将内部贴图路径解析为线性 handle
        app.init_asset::<ConfigVfx>();
        app.init_asset_loader::<crate::loaders::vfx::ConfigVfxLoader>();
        app.register_type::<ConfigVfxHandle>();

        app.register_type::<ShaderMap>();
        app.register_type::<lol_base_render::shader::ShaderMapEntry>();
        app.register_type::<lol_base_render::shader::SharedRenderData>();
        app.register_type::<lol_base_render::shader::SharedSamplerDef>();
        app.register_type::<lol_base_render::shader::SharedTextureDef>();
        app.register_type::<LeagueShader>();
        app.register_type::<shader_layout::ShaderMemberLayout>();
        app.register_type::<shader_layout::BindingTypeDesc>();
        app.register_type::<shader_layout::BindingDescriptor>();
        app.register_type::<shader_layout::ShaderLayoutDescriptor>();
        app.init_asset::<ConfigVfxSystemDefinition>();
        app.init_asset::<ConfigResourceResolver>();

        app.add_systems(PostUpdate, inject_vfx_assets);

        app.add_plugins(PluginDynamicMaterial);

        app.init_resource::<ParticleMesh>();
        app.init_resource::<DebugShaderHandles>();
        app.init_resource::<ResourceCache>();

        app.add_systems(Startup, startup_load_shaders);

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
                update_emitters,
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

    // 因为发射器 Transform 后续每帧由 update_emitter_position 覆写，所以
    // 朝向覆盖除存进发射器状态外，也要写进初始变换，避免首帧朝向错误
    let mut global_transform = global_transform;
    if let Some(rotation) = trigger.rotation {
        global_transform.rotation = rotation;
    }
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

    // 自动播放粒子系统关联的创建音效（soundOnCreateDefault）
    if let Some(sound_name) = &vfx_system_def.sound_on_create_default {
        if !sound_name.is_empty() {
            commands.trigger(CommandSkinSoundPlay {
                entity,
                key: sound_name.clone(),
            });
        }
    }

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

    for (i, vfx_emitter_definition_data) in vfx_emitter_definition_datas.enumerate() {
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
            ParticleEmitterState::new(
                vfx_emitter_definition_data,
                global_transform,
                trigger.rotation,
            ),
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

/// 当 ConfigVfx 资产及其依赖（粒子贴图）全部加载完成后，将其中所有 system/resolver 注入到对应的 Assets 中，
/// 使 ParticleId::get_def 能通过 system_hash 直接查到定义。
/// 因为 ConfigVfx 已改为 Asset，所以改由 AssetEvent::LoadedWithDependencies 驱动，重复注入幂等。
fn inject_vfx_assets(
    mut events: MessageReader<AssetEvent<ConfigVfx>>,
    res_assets_vfx: Res<Assets<ConfigVfx>>,
    mut res_assets_vfx_system: ResMut<Assets<ConfigVfxSystemDefinition>>,
    mut res_assets_vfx_resolver: ResMut<Assets<ConfigResourceResolver>>,
) {
    for event in events.read() {
        let (AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id }) = event
        else {
            continue;
        };
        let Some(config_vfx) = res_assets_vfx.get(*id) else {
            continue;
        };
        info!(
            "注入 {} 个 system 和 {} 个 resolver",
            config_vfx.systems.len(),
            config_vfx.resolvers.len(),
        );
        for (&hash, system_def) in &config_vfx.systems {
            res_assets_vfx_system.add_hash(hash, system_def.clone());
        }
        for (&hash, resolver) in &config_vfx.resolvers {
            res_assets_vfx_resolver.add_hash(hash, resolver.clone());
        }
    }
}
