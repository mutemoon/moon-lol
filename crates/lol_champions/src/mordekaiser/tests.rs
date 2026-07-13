#![cfg(test)]

//! 莫德凯撒集成测试。当前为框架阶段，仅含冒烟测试；
//! 各技能行为测试待 TDD 阶段在对应 `*_tests.rs` 中补充。

use crate::mordekaiser::Mordekaiser;
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

/// 框架冒烟测试：莫德凯撒能被正常构造并加载配置。
#[test]
fn mordekaiser_smoke_spawn() {
    let mut h = build_headless("morde_smoke");
    let pos = h.position(h.champion);
    assert!(pos.x.is_finite() && pos.y.is_finite() && pos.z.is_finite());
    h.finish();
}
