#![cfg(test)]

use bevy::math::{Vec2, Vec3};
use lol_core::attack::EventAttackEnd;
use lol_core::movement::{CastBlock, MovementBlock};

use crate::garen::Garen;
use crate::test_utils::*;

pub fn garen_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "garen",
        config_path: "characters/Garen/config.ron",
        skin_path: "characters/Garen/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::garen::PluginGaren);
        },
    }
}

pub fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Garen>(name, HarnessMode::Headless, &garen_config())
}

/// Garen Q 强化普攻命中应沉默目标（CastBlock），且不定身（无 MovementBlock）。
#[test]
fn garen_q_silences_target_on_attack() {
    let mut h = build_headless("garen_q_silence");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    // 施放 Q 获得 BuffGarenQAttack（强化下次普攻）
    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.1);

    // 模拟强化普攻命中
    h.app
        .world_mut()
        .entity_mut(h.champion)
        .trigger(|e| EventAttackEnd {
            entity: e,
            target: enemy,
        });
    h.advance(0.2);

    // 系统只认标记：沉默 -> CastBlock；沉默不定身 -> 无 MovementBlock
    assert!(
        h.app.world().get::<CastBlock>(enemy).is_some(),
        "Garen Q 命中应沉默目标（CastBlock）"
    );
    assert!(
        h.app.world().get::<MovementBlock>(enemy).is_none(),
        "沉默不应定身（无 MovementBlock）"
    );

    h.finish();
}
