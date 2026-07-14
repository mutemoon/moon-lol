#![cfg(test)]

use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::{DebuffStun, ImmuneToCC};
use lol_core::movement::MovementBlock;

use crate::olaf::Olaf;
use crate::test_utils::*;

pub fn olaf_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "olaf",
        config_path: "characters/Olaf/config.ron",
        skin_path: "characters/Olaf/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::olaf::PluginOlaf);
        },
    }
}

pub fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Olaf>(name, HarnessMode::Headless, &olaf_config())
}

/// 对角色施加眩晕（独立 buff 实体），推进一帧让观察者桥接标记。
fn stun(app: &mut App, char: Entity) {
    app.world_mut()
        .entity_mut(char)
        .with_related::<BuffOf>(DebuffStun::new(2.0));
    app.update();
}

/// Olaf R 期间免疫新控制：施加的眩晕不沾身（无 MovementBlock）。
#[test]
fn olaf_r_blocks_new_cc() {
    let mut h = build_headless("olaf_r_block");
    let olaf = h.champion;

    // 先开 R（未被控时可正常施放）-> 进入免控
    h.cast_skill(3, Vec2::ZERO).advance(0.2);
    assert!(
        h.app.world().get::<ImmuneToCC>(olaf).is_some(),
        "R 释放后应有 ImmuneToCC"
    );

    // 免控期间施加眩晕：CC 不沾身
    stun(&mut h.app, olaf);
    assert!(
        h.app.world().get::<MovementBlock>(olaf).is_none(),
        "免控期间新施加的 CC 不应沾身"
    );

    h.finish();
}

/// Olaf R 过期后恢复可被控制。
#[test]
fn olaf_r_expires_restores_vulnerability() {
    let mut h = build_headless("olaf_r_expire");
    let olaf = h.champion;

    h.cast_skill(3, Vec2::ZERO).advance(0.2);
    assert!(h.app.world().get::<ImmuneToCC>(olaf).is_some());

    // 推进超过 R 持续时间（6s），BuffOlafR 过期 -> sync 移除 ImmuneToCC
    h.advance(6.5);
    assert!(
        h.app.world().get::<ImmuneToCC>(olaf).is_none(),
        "R 过期后 ImmuneToCC 应移除"
    );

    // 恢复可被控制
    stun(&mut h.app, olaf);
    assert!(
        h.app.world().get::<MovementBlock>(olaf).is_some(),
        "R 过期后施加的眩晕应生效"
    );

    let _ = Vec3::ZERO;
    h.finish();
}
