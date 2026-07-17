#![cfg(test)]
//! Logic and render/video tests for Fiora skills.

use crate::fiora::Fiora;
use crate::test_utils::*;

pub fn fiora_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "fiora",
        config_path: "characters/fiora/config.ron",
        skin_path: "characters/fiora/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::fiora::PluginFiora);
        },
    }
}

pub fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Fiora>(name, HarnessMode::Headless, &fiora_config())
}

pub fn build_render(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Fiora>(name, HarnessMode::Render, &fiora_config())
}
