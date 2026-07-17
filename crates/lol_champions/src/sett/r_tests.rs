#![cfg(test)]

//! Sett R（消防官）集成测试（TDD）。
//!
//! R：在施法点 AoE 砸地（半径 200），造成 base+1.2×AD 物理伤害 + 40% 减速 1.5s。
//! Sett 基础 AD=60，1 级伤害 = 100+1.2×60 = 172。（位移/搬运简化为定点 AoE）

use bevy::math::{Vec2, Vec3};

use super::tests::{build_headless, is_slowed};

/// 施法点内的敌人应受到 172 物理伤害（被抓取后拖到落地）。
#[test]
fn sett_r_damages_at_cast_point() {
    let mut h = build_headless("sett_r_dmg");
    // 敌人在前方 300，在抓取范围 475 内 → 被抱起拖到落地
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.5);

    let dealt = hp_before - h.health(enemy);
    assert!(
        (dealt - 172.0).abs() < 1.0,
        "R 应在落地造成 100+1.2*AD=172 物理伤害，实际 {}",
        dealt
    );
    h.finish();
}

/// 命中敌人应被减速 40%/1.5s（被抓取后 AoE 命中）。
#[test]
fn sett_r_slows_hit_enemies() {
    let mut h = build_headless("sett_r_slow");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.5);

    assert!(is_slowed(&h, enemy), "R 命中敌人应被减速 40%/1.5s");
    h.finish();
}

/// 超出抓取范围 + 落地 AoE 半径的敌人不应被命中。
#[test]
fn sett_r_misses_outside_range() {
    let mut h = build_headless("sett_r_miss");
    // 敌人在 600，超出抓取范围 475 且超出落地 AoE 半径 200
    let enemy = h.add_enemy(Vec3::new(600.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    // 朝 0 方向施法，落点在 0，敌人距落点 600 > 200
    h.cast_skill(3, Vec2::new(0.0, 0.0)).advance(0.5);

    let dealt = hp_before - h.health(enemy);
    assert!(
        dealt.abs() < 0.5,
        "超出范围敌人不应被 R 命中，实际 {}",
        dealt
    );
    h.finish();
}
