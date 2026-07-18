#![cfg(test)]

use bevy::math::Vec2;
use bevy::prelude::*;
use lol_core::movement::{CastBlock, MovementBlock};

use crate::irelia::buffs::DebuffIreliaUnsteady;
use crate::irelia::tests::*;

/// E2 命中：造成 total_damage 魔法伤害 + 眩晕 + 不稳标记。
#[test]
fn irelia_e2_damages_stuns_and_marks() {
    let mut h = build_headless("irelia_e2_hit");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    let ad = ad(&h);
    let expected = h
        .get_skill_value(
            2,
            "total_damage",
            1,
            |stat| if stat == 2 { ad } else { 0.0 },
        )
        .unwrap();

    h.cast_skill(2, Vec2::new(300.0, 0.0));
    h.advance(0.1);
    h.cast_skill(2, Vec2::new(300.0, 0.0));
    h.advance(0.2);

    // 伤害
    assert!(
        (h.health(enemy) - (6000.0 - expected)).abs() < 1.0,
        "E2 应造成 {expected} 伤害，实际剩余 {}",
        h.health(enemy)
    );
    // 眩晕
    assert!(
        h.app.world().get::<MovementBlock>(enemy).is_some(),
        "E2 应眩晕敌人（MovementBlock）"
    );
    assert!(
        h.app.world().get::<CastBlock>(enemy).is_some(),
        "E2 应眩晕敌人（CastBlock）"
    );
    // 不稳标记
    assert!(
        has_buff::<DebuffIreliaUnsteady>(&h, enemy),
        "E2 应标记敌人不稳"
    );

    // 0.75s 眩晕过期，但 5s 标记仍在
    h.advance(0.9);
    assert!(
        h.app.world().get::<MovementBlock>(enemy).is_none(),
        "眩晕过期后 MovementBlock 应清除"
    );
    assert!(
        h.app.world().get::<CastBlock>(enemy).is_none(),
        "眩晕过期后 CastBlock 应清除"
    );
    assert!(
        has_buff::<DebuffIreliaUnsteady>(&h, enemy),
        "眩晕过期后不稳标记（5s）应仍存在"
    );
    h.finish();
}

/// E1 仅开启重施窗口，不造成伤害也不施加控制。
#[test]
fn irelia_e1_only_opens_recast_window() {
    let mut h = build_headless("irelia_e1_recast");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let skill_e = h.skill_entity(2);
    let mana_before = h.mana();

    h.cast_skill(2, Vec2::new(300.0, 0.0));
    h.advance(0.1);

    assert_eq!(
        h.recast_window_stage(skill_e),
        Some(2),
        "E1 后应进入 stage 2 重施窗口"
    );
    assert!((h.health(enemy) - 6000.0).abs() < 0.01, "E1 不应造成伤害");
    assert!(
        h.app.world().get::<MovementBlock>(enemy).is_none(),
        "E1 不应眩晕"
    );
    assert!(h.mana() < mana_before, "E1 施放应消耗法力");

    // E1 开启 4s 重施窗口，窗口过期后进入 16s 冷却
    h.advance(4.5);
    assert!(!h.can_cast(2), "E1 重施窗口过期后应进入冷却");
    h.finish();
}
