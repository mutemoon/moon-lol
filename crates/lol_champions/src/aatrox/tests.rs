#![cfg(test)]

use bevy::prelude::*;
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::Buffs;
use lol_core::damage::Damage;
use lol_core::life::Health;
use lol_core::skill::Skill;

use crate::aatrox::Aatrox;
use crate::test_utils::{ChampionHarnessConfig, ChampionTestHarness, HarnessMode};

pub fn aatrox_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "aatrox",
        config_path: "characters/aatrox/config.ron",
        skin_path: "characters/aatrox/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::aatrox::PluginAatrox);
        },
    }
}

pub fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Aatrox>(name, HarnessMode::Headless, &aatrox_config())
}

/// 直接把指定技能槽升到 `level`（harness 默认把所有技能升到 1）。
pub fn level_skill(h: &mut ChampionTestHarness, index: usize, level: usize) {
    let skill_entity = h.skill_entity(index);
    if let Some(mut skill) = h.app.world_mut().get_mut::<Skill>(skill_entity) {
        skill.level = level;
    }
}

/// 触发一次普攻命中事件（无头模式下不模拟基础攻击伤害，仅驱动 on-attack-end 观察者）。
pub fn attack_end(h: &mut ChampionTestHarness, target: Entity) {
    h.app.world_mut().trigger(EventAttackEnd {
        entity: h.champion,
        target,
    });
}

pub fn max_health(h: &ChampionTestHarness, entity: Entity) -> f32 {
    h.app
        .world()
        .get::<Health>(entity)
        .map(|hp| hp.max)
        .unwrap_or(0.0)
}

pub fn ad(h: &ChampionTestHarness) -> f32 {
    h.app
        .world()
        .get::<Damage>(h.champion)
        .map(|d| d.0)
        .unwrap_or(0.0)
}

pub fn is_knockup(h: &ChampionTestHarness, entity: Entity) -> bool {
    has_debuff::<lol_core::buffs::cc_debuffs::DebuffKnockup>(h, entity)
}

pub fn is_slowed(h: &ChampionTestHarness, entity: Entity) -> bool {
    has_debuff::<lol_core::buffs::cc_debuffs::DebuffSlow>(h, entity)
}

pub fn has_mark(h: &ChampionTestHarness, entity: Entity) -> bool {
    has_debuff::<crate::aatrox::buffs::DebuffAatroxWMark>(h, entity)
}

pub fn has_movespeed_buff(h: &ChampionTestHarness, entity: Entity) -> bool {
    has_debuff::<lol_core::buffs::common_buffs::BuffMoveSpeed>(h, entity)
}

fn has_debuff<T: Component>(h: &ChampionTestHarness, entity: Entity) -> bool {
    let Some(buffs) = h.app.world().get::<Buffs>(entity) else {
        return false;
    };
    for buff_entity in buffs.iter() {
        if h.app.world().get::<T>(buff_entity).is_some() {
            return true;
        }
    }
    false
}
