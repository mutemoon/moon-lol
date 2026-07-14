#![cfg(test)]

use crate::camille::Camille;
use crate::test_utils::*;

pub fn camille_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "camille",
        config_path: "characters/camille/config.ron",
        skin_path: "characters/camille/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::camille::PluginCamille);
        },
    }
}

pub fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Camille>(name, HarnessMode::Headless, &camille_config())
}
