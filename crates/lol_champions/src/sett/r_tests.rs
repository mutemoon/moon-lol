#![cfg(test)]

//! Sett R（消防官）集成测试（TDD）。
//!
//! R：在施法点 AoE 砸地（半径 200），造成 base+1.2×AD 物理伤害 + 40% 减速 1.5s。
//! Sett 基础 AD=60，1 级伤害 = 100+1.2×60 = 172。（位移/搬运简化为定点 AoE）

use bevy::math::{Vec2, Vec3};

use super::tests::{build_headless, is_slowed};

/// 施法点内的敌人应受到 172 物理伤害。
#[test]
fn sett_r_damages_at_cast_point() {
    let mut h = build_headless("sett_r_dmg");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.3);

    let dealt = hp_before - h.health(enemy);
    assert!(
        (dealt - 172.0).abs() < 1.0,
        "R 应在施法点造成 100+1.2*AD=172 物理伤害，实际 {}",
        dealt
    );
    h.finish();
}

/// 命中敌人应被减速 40%/1.5s。
#[test]
fn sett_r_slows_hit_enemies() {
    let mut h = build_headless("sett_r_slow");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.3);

    assert!(is_slowed(&h, enemy), "R 命中敌人应被减速 40%/1.5s");
    h.finish();
}

/// 施法点远离敌人（距离 > 半径 200）时不应命中。
#[test]
fn sett_r_misses_outside_radius() {
    let mut h = build_headless("sett_r_miss");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    h.cast_skill(3, Vec2::new(600.0, 0.0)).advance(0.3);

    let dealt = hp_before - h.health(enemy);
    assert!(
        dealt.abs() < 0.5,
        "施法点 600 远离敌人（300），距离 300 > 半径 200，不应命中，实际 {}",
        dealt
    );
    h.finish();
}
