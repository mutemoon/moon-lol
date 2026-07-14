#![cfg(test)]

use bevy::prelude::*;
use lol_core::base::buff::Buffs;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};

use crate::irelia::Irelia;
use crate::irelia::passive::BuffIreliaFervor;
use crate::test_utils::*;

pub fn irelia_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "irelia",
        config_path: "characters/Irelia/config.ron",
        skin_path: "characters/Irelia/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::irelia::PluginIrelia);
        },
    }
}

pub fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Irelia>(name, HarnessMode::Headless, &irelia_config())
}

/// Irelia 的当前攻击力（Damage 组件）。
pub fn ad(h: &ChampionTestHarness) -> f32 {
    h.app
        .world()
        .get::<Damage>(h.champion)
        .map(|d| d.0)
        .unwrap_or(0.0)
}

/// 读取 Irelia 艾欧尼亚热诚的当前层数（无 buff 返回 None）。
pub fn fervor_charges(h: &ChampionTestHarness) -> Option<u8> {
    let buffs = h.app.world().get::<Buffs>(h.champion)?;
    for b in buffs.iter() {
        if let Some(f) = h.app.world().get::<BuffIreliaFervor>(b) {
            return Some(f.charges);
        }
    }
    None
}

/// 判断某实体身上是否存在指定类型的子 buff。
pub fn has_buff<T: Component>(h: &ChampionTestHarness, entity: Entity) -> bool {
    let Some(buffs) = h.app.world().get::<Buffs>(entity) else {
        return false;
    };
    buffs.iter().any(|b| h.app.world().get::<T>(b).is_some())
}

/// 对 Irelia 施加魔法伤害（绕过护甲，专用于测试减伤 buff）。
/// 物理伤害会被 Irelia 的护甲削减，干扰减伤断言；魔法伤害不经过护甲，减伤仍生效。
pub fn apply_magic_damage(h: &mut ChampionTestHarness, source: Entity, amount: f32) {
    let target = h.champion;
    h.app.world_mut().trigger(CommandDamageCreate {
        entity: target,
        source,
        damage_type: DamageType::Magic,
        amount,
        tag: None,
    });
}
