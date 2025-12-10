use bevy::prelude::*;
use league_core::{ResourceResolver, SkinCharacterDataProperties};
use lol_config::LeagueProperties;

use crate::{CommandParticleDespawn, CommandParticleSpawn, Skin};

#[derive(EntityEvent)]
pub struct CommandSkinParticleSpawn {
    pub entity: Entity,
    pub hash: u32,
}

#[derive(EntityEvent)]
pub struct CommandSkinParticleDespawn {
    pub entity: Entity,
    pub hash: u32,
}

fn resolve_skin_resource_record<'a>(
    entity: Entity,
    input_hash: u32, // 假设 trigger.hash 是 u32，请根据实际类型调整
    query_skin: &Query<&Skin>,
    league_props: &LeagueProperties,
    assets_skin: &Assets<SkinCharacterDataProperties>,
    assets_resolver: &'a Assets<ResourceResolver>,
) -> Option<&'a u32> {
    // 假设 record 是 u32 或类似的引用
    // 1. 获取 Skin 组件
    let skin = query_skin.get(entity).ok()?;

    // 2. 获取角色皮肤数据
    let skin_data = league_props.get(assets_skin, skin.key)?;

    // 3. 获取资源解析器句柄
    let resolver_handle = skin_data.m_resource_resolver?;

    // 4. 获取资源解析器资产
    let resolver = league_props.get(assets_resolver, resolver_handle)?;

    // 5. 查找具体的资源记录
    resolver.resource_map.as_ref()?.get(&input_hash)
}

pub fn on_command_character_particle_spawn(
    trigger: On<CommandSkinParticleSpawn>,
    res_assets_resource_resolver: Res<Assets<ResourceResolver>>,
    res_assets_skin_character_data_properties: Res<Assets<SkinCharacterDataProperties>>,
    res_league_properties: Res<LeagueProperties>,
    mut commands: Commands,
    query: Query<&Skin>,
) {
    let entity = trigger.event_target();

    // 使用辅助函数获取记录，如果任何一步失败（返回 None），直接 return
    let Some(record) = resolve_skin_resource_record(
        entity,
        trigger.hash,
        &query,
        &res_league_properties,
        &res_assets_skin_character_data_properties,
        &res_assets_resource_resolver,
    ) else {
        return;
    };

    commands.trigger(CommandParticleSpawn {
        entity,
        hash: (*record).into(), // 解引用并转换
    });
}

pub fn on_command_character_particle_despawn(
    trigger: On<CommandSkinParticleDespawn>,
    res_assets_resource_resolver: Res<Assets<ResourceResolver>>,
    res_assets_skin_character_data_properties: Res<Assets<SkinCharacterDataProperties>>,
    res_league_properties: Res<LeagueProperties>,
    mut commands: Commands,
    query: Query<&Skin>,
) {
    let entity = trigger.event_target();

    // 复用相同的逻辑
    let Some(record) = resolve_skin_resource_record(
        entity,
        trigger.hash,
        &query,
        &res_league_properties,
        &res_assets_skin_character_data_properties,
        &res_assets_resource_resolver,
    ) else {
        return;
    };

    commands.trigger(CommandParticleDespawn {
        entity,
        hash: (*record).into(),
    });
}
