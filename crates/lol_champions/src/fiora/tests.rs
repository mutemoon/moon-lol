#![cfg(test)]
//! Logic and render/video tests for Fiora skills.

use bevy::math::Vec2;

use crate::fiora::Fiora;
use crate::test_utils::*;

fn fiora_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "fiora",
        config_path: "characters/fiora/config.ron",
        skin_path: "characters/fiora/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::fiora::PluginFiora);
        },
    }
}

fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Fiora>(name, HarnessMode::Headless, &fiora_config())
}

fn build_render(name: &str, max_frames: u32) -> ChampionTestHarness {
    ChampionTestHarness::build::<Fiora>(name, HarnessMode::Render { max_frames }, &fiora_config())
}

#[test]
fn fiora_harness_builds_and_settles() {
    let _h = build_headless("fiora_smoke");
}

#[test]
fn fiora_q_writes_video() {
    let mut h = build_render("fiora_q_writes_video", 120);
    h.cast_skill(0, Vec2::new(170.0, 0.0)).advance(0.4);
    h.finish();
}

#[test]
fn fiora_w_writes_video() {
    let mut h = build_render("fiora_w_writes_video", 120);
    h.cast_skill(1, Vec2::new(170.0, 0.0)).advance(0.4);
    h.finish();
}

#[test]
fn fiora_e_writes_video() {
    let mut h = build_render("fiora_e_writes_video", 120);
    h.cast_skill(2, Vec2::new(170.0, 0.0)).advance(0.4);
    h.finish();
}

#[test]
fn fiora_r_writes_video() {
    let mut h = build_render("fiora_r_writes_video", 140);
    h.cast_skill(3, Vec2::new(170.0, 0.0)).advance(0.4);
    h.finish();
}
