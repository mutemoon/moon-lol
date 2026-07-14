#![cfg(test)]

use bevy::prelude::*;
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::Buffs;
use lol_core::buffs::shield_white::BuffShieldWhite;

use crate::sett::Sett;
use crate::test_utils::*;

pub fn sett_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "sett",
        config_path: "characters/sett/config.ron",
        skin_path: "characters/sett/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::sett::PluginSett);
        },
    }
}

pub fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Sett>(name, HarnessMode::Headless, &sett_config())
}

/// 将指定技能槽等级提升到指定值（0=Q,1=W,2=E,3=R）。
pub fn level_skill(h: &mut ChampionTestHarness, index: usize, level: usize) {
    let skill_entity = h.skill_entity(index);
    if let Some(mut skill) = h
        .app
        .world_mut()
        .get_mut::<lol_core::skill::Skill>(skill_entity)
    {
        skill.level = level;
    }
}

/// 手动触发一次普攻命中（仅触发 on-hit，不造成基础 AA 伤害）。
pub fn attack_end(h: &mut ChampionTestHarness, target: Entity) {
    h.app.world_mut().trigger(EventAttackEnd {
        entity: h.champion,
        target,
    });
}

/// 读取 Sett 当前储存的灰心值。
pub fn grit_value(h: &ChampionTestHarness) -> f32 {
    h.app
        .world()
        .get::<crate::sett::buffs::SettGrit>(h.champion)
        .map(|g| g.stored)
        .unwrap_or(0.0)
}

/// 敌人是否挂有眩晕。
pub fn is_stunned(h: &ChampionTestHarness, entity: Entity) -> bool {
    has_debuff::<lol_core::buffs::cc_debuffs::DebuffStun>(h, entity)
}

/// 敌人是否挂有减速。
pub fn is_slowed(h: &ChampionTestHarness, entity: Entity) -> bool {
    has_debuff::<lol_core::buffs::cc_debuffs::DebuffSlow>(h, entity)
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

/// 读取 Sett 白盾当前值（透传 harness.shield_value，便于 sett 测试就近调用）。
#[allow(dead_code)]
pub fn sett_shield(h: &ChampionTestHarness) -> Option<f32> {
    let buffs = h.app.world().get::<Buffs>(h.champion)?;
    for buff in buffs.iter() {
        if let Some(shield) = h.app.world().get::<BuffShieldWhite>(buff) {
            return Some(shield.current);
        }
    }
    None
}
