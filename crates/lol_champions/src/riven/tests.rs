#![cfg(test)]

//! Riven integration tests that span multiple skills or test cross-skill interactions.
//! Individual skill tests are in their respective *_tests.rs files.

use bevy::math::{Vec2, Vec3};
use lol_core::movement::MovementBlock;

use crate::riven::{BuffStun, Riven};
use crate::test_utils::*;

const EPSILON: f32 = 1e-3;

pub fn riven_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "riven",
        config_path: "characters/Riven/config.ron",
        skin_path: "characters/Riven/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::riven::PluginRiven);
        },
    }
}

pub fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Riven>(name, HarnessMode::Headless, &riven_config())
}

pub fn build_render(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Riven>(name, HarnessMode::Render, &riven_config())
}

/// Tests that stun prevents skill casting - cross-skill interaction test.
#[test]
fn riven_stun_prevents_skill_cast() {
    let mut h = build_headless("riven_stun_block");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    // E 后 W（不应被眩晕阻挡，先让 Riven 靠近）
    h.cast_skill(2, Vec2::new(300.0, 0.0)).advance(0.3);
    // W 敌人，敌人被眩晕
    h.cast_skill(1, Vec2::new(300.0, 0.0)).advance(0.1);

    // 敌人在眩晕中不应该能施放技能……但没有敌人技能系统可以测试
    // 而是通过 Riven 自身模拟：给自己加一个眩晕
    h.app.world_mut().entity_mut(h.champion).insert(BuffStun {
        timer: bevy::prelude::Timer::from_seconds(10.0, bevy::prelude::TimerMode::Once),
    });
    h.app
        .world_mut()
        .entity_mut(h.champion)
        .insert(MovementBlock);

    let before_cast_hp = h.health(enemy);
    // 眩晕中尝试施放 Q，应被阻止
    h.cast_skill(0, Vec2::new(300.0, 0.0)).advance(0.3);

    assert!(
        (h.health(enemy) - before_cast_hp).abs() < EPSILON,
        "眩晕中不应能施放技能造成伤害"
    );

    h.finish();
}
