#![cfg(test)]

use bevy::prelude::*;
use lol_core::attack::{BuffAttack, EventAttackEnd};
use lol_core::base::buff::Buffs;
use lol_core::life::Health;

use crate::test_utils::*;
use crate::volibear::Volibear;

pub fn volibear_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "volibear",
        config_path: "characters/volibear/config.ron",
        skin_path: "characters/volibear/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::volibear::PluginVolibear);
        },
    }
}

pub fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Volibear>(name, HarnessMode::Headless, &volibear_config())
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

/// 手动触发一次普攻命中（仅触发 on-hit/被动，不造成基础 AA 伤害）。
/// 触发后推进 1 帧以刷新 deferred commands，使 BuffAttack 实体生效。
pub fn attack_end(h: &mut ChampionTestHarness, target: Entity) {
    h.app.world_mut().trigger(EventAttackEnd {
        entity: h.champion,
        target,
    });
    h.app.update();
}

/// 读取英雄当前来自 buff 的额外攻击速度（被动层数 × 每层比例）。
/// 优先查 BuffAttack 实体（通过 Buffs 关系），其次查 champion 自身。
pub fn attack_speed_bonus(h: &ChampionTestHarness) -> f32 {
    // 先从 buff 实体查找
    if let Some(buffs) = h.app.world().get::<Buffs>(h.champion) {
        let sum: f32 = buffs
            .iter()
            .filter_map(|e| h.app.world().get::<BuffAttack>(e))
            .map(|b| b.bonus_attack_speed)
            .sum();
        if sum > 0.0 {
            return sum;
        }
    }
    // 再查 champion 自身（直接 insert 模式）
    h.app
        .world()
        .get::<BuffAttack>(h.champion)
        .map(|b| b.bonus_attack_speed)
        .unwrap_or(0.0)
}

/// 读取实体最大生命值。
pub fn max_health(h: &ChampionTestHarness, entity: Entity) -> f32 {
    h.app
        .world()
        .get::<Health>(entity)
        .map(|hp| hp.max)
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

/// 敌人是否挂有沃利贝尔 W 标记。
pub fn has_mark(h: &ChampionTestHarness, entity: Entity) -> bool {
    has_debuff::<crate::volibear::buffs::DebuffVolibearWMark>(h, entity)
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
