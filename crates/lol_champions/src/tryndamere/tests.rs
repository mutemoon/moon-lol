#![cfg(test)]

use bevy::math::{Vec2, Vec3};
use lol_core::damage::{CommandDamageCreate, DamageType};
use lol_core::movement::MovementSlow;

use crate::test_utils::*;
use crate::tryndamere::Tryndamere;

pub fn tryndamere_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "tryndamere",
        config_path: "characters/Tryndamere/config.ron",
        skin_path: "characters/Tryndamere/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::tryndamere::PluginTryndamere);
        },
    }
}

pub fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Tryndamere>(name, HarnessMode::Headless, &tryndamere_config())
}

/// Tryndamere 伤害命中应施加减速（MovementSlow），过期自动清除。
#[test]
fn tryndamere_damage_hit_slows_enemy() {
    let mut h = build_headless("tryndamere_slow");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));

    h.app
        .world_mut()
        .entity_mut(enemy)
        .trigger(|e| CommandDamageCreate {
            entity: e,
            source: h.champion,
            damage_type: DamageType::Physical,
            amount: 10.0,
            tag: None,
        });
    h.advance(0.2);

    let slow = h
        .app
        .world()
        .get::<MovementSlow>(enemy)
        .expect("应有 MovementSlow");
    assert!(
        (slow.percent - 0.35).abs() < 1e-3,
        "减速比例应为 0.35，实际 {}",
        slow.percent
    );

    // 2s 后减速过期（标记随 buff 死亡自动清除）
    h.advance(2.2);
    assert!(
        h.app.world().get::<MovementSlow>(enemy).is_none(),
        "2s 后减速应过期"
    );

    let _ = Vec2::ZERO;
    h.finish();
}
