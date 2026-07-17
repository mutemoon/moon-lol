#![cfg(test)]

//! 莫德凯撒集成测试公共夹具与辅助函数。

use bevy::ecs::entity::Entity;
use bevy::math::Vec2;
use lol_core::damage::{AbilityPower, Armor, CommandDamageCreate, Damage, DamageType};
use lol_core::life::Health;
use lol_core::movement::Movement;

use crate::mordekaiser::Mordekaiser;
use crate::mordekaiser::buffs::{MordekaiserRealm, MordekaiserStatSteal, MordekaiserWStorage};
use crate::mordekaiser::passive::MordekaiserDarkness;
use crate::test_utils::*;

pub fn mordekaiser_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "mordekaiser",
        config_path: "characters/Mordekaiser/config.ron",
        skin_path: "characters/Mordekaiser/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::mordekaiser::PluginMordekaiser);
        },
    }
}

pub fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Mordekaiser>(name, HarnessMode::Headless, &mordekaiser_config())
}

// ── 属性注入 / 读取 ──

/// 注入法术强度（莫德凯撒基础 AP 为 0，需由测试注入以验证 AP 加成）。
pub fn give_ap(h: &mut ChampionTestHarness, ap: f32) {
    h.app
        .world_mut()
        .entity_mut(h.champion)
        .insert(AbilityPower(ap));
}

pub fn morde_ad(h: &ChampionTestHarness) -> f32 {
    h.app
        .world()
        .get::<Damage>(h.champion)
        .map(|d| d.0)
        .unwrap_or(0.0)
}

pub fn morde_armor(h: &ChampionTestHarness) -> f32 {
    h.app
        .world()
        .get::<Armor>(h.champion)
        .map(|a| a.0)
        .unwrap_or(0.0)
}

pub fn morde_max_hp(h: &ChampionTestHarness) -> f32 {
    h.app
        .world()
        .get::<Health>(h.champion)
        .map(|h| h.max)
        .unwrap_or(0.0)
}

pub fn morde_speed(h: &ChampionTestHarness) -> f32 {
    h.app
        .world()
        .get::<Movement>(h.champion)
        .map(|m| m.speed)
        .unwrap_or(0.0)
}

// ── 伤害触发 ──

/// 莫德凯撒对敌人造成一次物理伤害（模拟普攻命中，触发被动叠层 + 附伤）。
pub fn morde_hit(h: &mut ChampionTestHarness, enemy: Entity, amount: f32) {
    let source = h.champion;
    h.app.world_mut().trigger(CommandDamageCreate {
        entity: enemy,
        source,
        damage_type: DamageType::Physical,
        amount,
        tag: None,
    });
}

/// 敌人对莫德凯撒造成一次物理伤害（触发 W 储存）。
pub fn morde_take_damage(h: &mut ChampionTestHarness, source: Entity, amount: f32) {
    h.app.world_mut().trigger(CommandDamageCreate {
        entity: h.champion,
        source,
        damage_type: DamageType::Physical,
        amount,
        tag: None,
    });
}

// ── 被动 / W / R 状态读取 ──

pub fn darkness_stacks(h: &ChampionTestHarness) -> Option<u8> {
    h.app
        .world()
        .get::<MordekaiserDarkness>(h.champion)
        .map(|d| d.stacks)
}

pub fn darkness_active(h: &ChampionTestHarness) -> bool {
    h.app
        .world()
        .get::<MordekaiserDarkness>(h.champion)
        .map(|d| d.active)
        .unwrap_or(false)
}

pub fn w_storage(h: &ChampionTestHarness) -> Option<f32> {
    h.app
        .world()
        .get::<MordekaiserWStorage>(h.champion)
        .map(|s| s.stored)
}

pub fn has_realm(h: &ChampionTestHarness) -> bool {
    h.app.world().get::<MordekaiserRealm>(h.champion).is_some()
}

pub fn realm_target(h: &ChampionTestHarness) -> Option<Entity> {
    h.app
        .world()
        .get::<MordekaiserRealm>(h.champion)
        .map(|r| r.target)
}

pub fn stat_steal(h: &ChampionTestHarness) -> Option<MordekaiserStatSteal> {
    h.app
        .world()
        .get::<MordekaiserStatSteal>(h.champion)
        .cloned()
}

/// 框架冒烟测试：莫德凯撒能被正常构造并加载配置。
#[test]
fn mordekaiser_smoke_spawn() {
    let mut h = build_headless("morde_smoke");
    let pos = h.position(h.champion);
    assert!(pos.x.is_finite() && pos.y.is_finite() && pos.z.is_finite());
    let _ = Vec2::ZERO;
    h.finish();
}
