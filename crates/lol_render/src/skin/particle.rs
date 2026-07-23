use bevy::prelude::*;
use lol_base::character::Skin;
use lol_base::hash_key::LoadHashKeyTrait;
use lol_base::particle::ConfigResourceResolver;
use lol_base::render_cmd::{CommandSkinParticleDespawn, CommandSkinParticleSpawn};

use crate::particle::{CommandParticleDespawn, CommandParticleSpawn};

pub fn on_command_character_particle_spawn(
    trigger: On<CommandSkinParticleSpawn>,
    mut commands: Commands,
    q_skin: Query<&Skin>,
    res_assets_resolver: Res<Assets<ConfigResourceResolver>>,
) {
    let entity = trigger.event_target();
    info!(
        "{entity} 收到皮肤粒子创建命令，trigger_key={}",
        trigger.hash
    );

    let Ok(skin) = q_skin.get(entity) else {
        info!("{entity} 找不到 Skin 组件，跳过粒子创建");
        return;
    };
    info!("{entity} Skin 组件 resolver_key={:08x}", skin.resolver_key);

    let Some(resolver) = res_assets_resolver.load_hash(skin.resolver_key) else {
        info!(
            "{entity} 找不到 ConfigResourceResolver(key={:08x})，跳过粒子创建；可能未提取 vfx.ron 或未重新提取皮肤",
            skin.resolver_key
        );
        return;
    };

    let Some(&vfx_hash) = resolver.resource_map.get(&trigger.hash) else {
        info!(
            "{entity} trigger_key={} 在 resolver 中找不到对应 vfx_hash，可用的 trigger_key 列表：{:?}",
            trigger.hash,
            resolver.resource_map.keys().collect::<Vec<_>>()
        );
        return;
    };
    info!("{entity} 解析到 vfx_hash={:08x}，触发粒子创建", vfx_hash);

    commands.trigger(CommandParticleSpawn {
        entity,
        vfx_handle: vfx_hash.into(),
    });
}

pub fn on_command_character_particle_despawn(
    trigger: On<CommandSkinParticleDespawn>,
    mut commands: Commands,
    q_skin: Query<&Skin>,
    res_assets_resolver: Res<Assets<ConfigResourceResolver>>,
) {
    let entity = trigger.event_target();
    info!(
        "{entity} 收到皮肤粒子销毁命令，trigger_key={}",
        trigger.hash
    );

    let Ok(skin) = q_skin.get(entity) else {
        info!("{entity} 找不到 Skin 组件，跳过粒子销毁");
        return;
    };

    let Some(resolver) = res_assets_resolver.load_hash(skin.resolver_key) else {
        info!(
            "{entity} 找不到 ConfigResourceResolver(key={:08x})，跳过粒子销毁",
            skin.resolver_key
        );
        return;
    };

    let Some(&vfx_hash) = resolver.resource_map.get(&trigger.hash) else {
        info!(
            "{entity} trigger_key={} 在 resolver 中找不到对应 vfx_hash",
            trigger.hash
        );
        return;
    };
    info!("{entity} 解析到 vfx_hash={:08x}，触发粒子销毁", vfx_hash);

    commands.trigger(CommandParticleDespawn {
        entity,
        vfx_handle: vfx_hash.into(),
    });
}
