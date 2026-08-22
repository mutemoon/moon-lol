use bevy::prelude::*;
use lol_base::character::Skin;
use lol_base::hash_key::LoadHashKeyTrait;
use lol_base::render_cmd::{CommandSkinParticleDespawn, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_base_render::particle::{
    CommandParticleDespawn, CommandParticleSpawn, ConfigResourceResolver,
};
use lol_core::attack::{Attack, EventAttackEnd};

/// 判断 resolver 触发名是否为普攻受击粒子（`*_tar`）。
///
/// 因为游戏数据中普攻受击粒子没有统一命名，只有约定俗成的变体
/// （`Fiora_BA1_tar`、`Zoe_BA_tar`、`Pantheon_BA_1_Tar`、`Xayah_BA_Hit_Tar`、
/// `Seraphine_BasicAttack_tar` 等），所以这里按分段匹配 `ba`/`ba{N}`/`basicattack`
/// 标记并排除暴击变体（crit）。
fn is_basic_attack_hit_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    if !key.ends_with("_tar") || key.contains("crit") {
        return false;
    }
    key.split('_').any(|seg| {
        seg == "basicattack"
            || seg == "ba"
            || (seg.len() > 2
                && seg.starts_with("ba")
                && seg[2..].chars().all(|c| c.is_ascii_digit()))
    })
}

/// 普攻命中时，用**攻击者**皮肤 resolver 中携带的 `*_tar` 受击粒子，
/// 在**受击者**身上生成粒子（如 Fiora 普攻命中 → 目标身上播 Fiora_BA1_tar）。
///
/// 因为 `_tar` 粒子只作为攻击者 resolver 的条目存在、不被任何数据显式引用
/// （见 docs/reverse/vfx-trigger.md），所以这里由命中事件按命名规律查表触发。
///
/// 因为普攻 spell 数据带 hit_bone_name（如 Fiora 的 C_BUFFBONE_GLB_CHEST_LOC），
/// 所以优先把粒子挂到受击者对应骨骼子实体上（同 missile.rs 的 end_entity 解析模式），
/// 找不到骨骼时回退到受击者根实体。
pub fn on_event_attack_end_spawn_hit_particle(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_skin: Query<&Skin>,
    q_attack: Query<&Attack>,
    q_children: Query<&Children>,
    q_name: Query<&Name>,
    res_assets_spell: Res<Assets<Spell>>,
    res_assets_resolver: Res<Assets<ConfigResourceResolver>>,
) {
    let attacker = trigger.event_target();
    let target = trigger.target;

    let Ok(skin) = q_skin.get(attacker) else {
        return;
    };

    let Some(resolver) = res_assets_resolver.load_hash(skin.resolver_key) else {
        return;
    };

    // BTreeMap 按键序遍历，取第一个匹配项保证同一英雄每次命中结果确定
    let Some((key, &vfx_hash)) = resolver
        .resource_map
        .iter()
        .find(|(key, _)| is_basic_attack_hit_key(key))
    else {
        return;
    };

    // 从攻击者普攻 spell 读 hit_bone_name，在受击者后代中按 Name 匹配骨骼实体
    let hit_bone_name = q_attack
        .get(attacker)
        .ok()
        .and_then(|attack| res_assets_spell.get(&attack.spell))
        .and_then(|spell| spell.spell_data.as_ref())
        .and_then(|data| data.hit_bone_name.clone());

    let mut anchor = target;
    if let Some(bone_name) = &hit_bone_name {
        for child in q_children.iter_descendants(target) {
            let Ok(name) = q_name.get(child) else {
                continue;
            };
            if name.as_str() == bone_name {
                anchor = child;
                break;
            }
        }
    }
    debug!(
        "{attacker} 普攻命中 {target}，在 {anchor}（骨骼={hit_bone_name:?}）播放受击粒子 {key}（vfx_hash={vfx_hash:08x}）"
    );

    commands.trigger(CommandParticleSpawn {
        entity: anchor,
        vfx_handle: vfx_hash.into(),
        rotation: None,
    });
}

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

    // 查表实体与挂载实体分离：“攻击者携带、播在目标身上”的粒子用
    // 攻击者的 resolver 解析，再挂到目标实体上
    let resolver_entity = trigger.resolver_entity.unwrap_or(entity);
    let Ok(skin) = q_skin.get(resolver_entity) else {
        info!("{resolver_entity} 找不到 Skin 组件，跳过粒子创建");
        return;
    };
    info!(
        "{resolver_entity} Skin 组件 resolver_key={:08x}",
        skin.resolver_key
    );

    let Some(resolver) = res_assets_resolver.load_hash(skin.resolver_key) else {
        info!(
            "{resolver_entity} 找不到 ConfigResourceResolver(key={:08x})，跳过粒子创建；可能 vfx 场景 skin{{N}}_vfx.ron 未提取或未加载（缺 ConfigVfx Resource）",
            skin.resolver_key
        );
        return;
    };

    let Some(&vfx_hash) = resolver.resource_map.get(&trigger.hash) else {
        info!(
            "{resolver_entity} trigger_key={} 在 resolver 中找不到对应 vfx_hash，可用的 trigger_key 列表：{:?}",
            trigger.hash,
            resolver.resource_map.keys().collect::<Vec<_>>()
        );
        return;
    };
    info!("{entity} 解析到 vfx_hash={:08x}，触发粒子创建", vfx_hash);

    commands.trigger(CommandParticleSpawn {
        entity,
        vfx_handle: vfx_hash.into(),
        rotation: trigger.rotation,
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

    // 与 spawn 同样支持查表实体分离，否则用攻击者 resolver 挂出的粒子无法撤销
    let resolver_entity = trigger.resolver_entity.unwrap_or(entity);
    let Ok(skin) = q_skin.get(resolver_entity) else {
        info!("{resolver_entity} 找不到 Skin 组件，跳过粒子销毁");
        return;
    };

    let Some(resolver) = res_assets_resolver.load_hash(skin.resolver_key) else {
        info!(
            "{resolver_entity} 找不到 ConfigResourceResolver(key={:08x})，跳过粒子销毁",
            skin.resolver_key
        );
        return;
    };

    let Some(&vfx_hash) = resolver.resource_map.get(&trigger.hash) else {
        info!(
            "{resolver_entity} trigger_key={} 在 resolver 中找不到对应 vfx_hash",
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
