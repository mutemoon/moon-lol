#![cfg(test)]

use bevy::math::Vec2;
use bevy::prelude::*;

use crate::irelia::tests::*;

/// Q 命中距施法点最近的敌方英雄，造成 ron `champion_damage` 物理伤害。
#[test]
fn irelia_q_damages_nearest_enemy() {
    let mut h = build_headless("irelia_q_damage");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let mana_before = h.mana();

    let ad = ad(&h);
    let expected = h
        .get_skill_value(
            0,
            "champion_damage",
            1,
            |stat| if stat == 2 { ad } else { 0.0 },
        )
        .unwrap();

    h.cast_skill(0, Vec2::new(300.0, 0.0));
    h.advance(0.2);

    let remaining = h.health(enemy);
    assert!(
        (remaining - (6000.0 - expected)).abs() < 1.0,
        "Q 应造成 {expected} 伤害，实际剩余血量 {remaining}"
    );
    assert!(!h.can_cast(0), "Q 施放后应进入冷却");
    assert!(h.mana() < mana_before, "Q 施放应消耗法力");
    h.finish();
}

/// Q 命中"不稳"标记的敌人时刷新自身冷却（核心追击机制）。
#[test]
fn irelia_q_refreshes_cooldown_on_unsteady_target() {
    let mut h = build_headless("irelia_q_refresh");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    // E1 -> E2：标记敌人不稳（E2 命中施加 DebuffIreliaUnsteady）
    h.cast_skill(2, Vec2::new(300.0, 0.0));
    h.advance(0.1);
    h.cast_skill(2, Vec2::new(300.0, 0.0));
    h.advance(0.1);
    assert!(
        has_buff::<crate::irelia::buffs::DebuffIreliaUnsteady>(&h, enemy),
        "E2 应标记敌人不稳"
    );

    // Q 命中不稳敌人 -> 冷却刷新（timer=None 立即就绪）
    h.cast_skill(0, Vec2::new(300.0, 0.0));
    h.advance(0.2);

    assert!(
        h.can_cast(0),
        "命中不稳敌人后 Q 冷却应被刷新，可立即再次施放"
    );
    h.finish();
}

/// Q 朝施法点冲刺位移（DashMoveType::Pointer）。
#[test]
fn irelia_q_dashes_toward_point() {
    let mut h = build_headless("irelia_q_dash");
    let _ = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    let start = h.position(h.champion);
    h.cast_skill(0, Vec2::new(300.0, 0.0));
    h.advance(0.5);

    let end = h.position(h.champion);
    assert!(
        end.x > start.x,
        "Q 应朝施法点冲刺，起点 x={} 终点 x={}",
        start.x,
        end.x
    );
    h.finish();
}
