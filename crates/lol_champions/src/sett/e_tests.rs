#![cfg(test)]

//! Sett E（迎面痛击）集成测试（TDD）。
//!
//! E：朝施法方向锥形拉回敌人到脚下（+击飞），造成 base+0.6×AD 物理伤害，
//!    并施加 0.5s 眩晕。Sett 基础 AD=60，1 级伤害 = 30+0.6×60 = 66。

use bevy::math::{Vec2, Vec3};

use super::tests::{build_headless, is_stunned};

/// 锥形内敌人应被拉到脚下并受到 66 物理伤害。
#[test]
fn sett_e_pulls_and_damages() {
    let mut h = build_headless("sett_e_pull_dmg");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    h.cast_skill(2, Vec2::new(800.0, 0.0)).advance(1.0);

    let dealt = hp_before - h.health(enemy);
    assert!(
        (dealt - 66.0).abs() < 1.0,
        "E 应造成 30+0.6*AD=66 物理伤害，实际 {}",
        dealt
    );
    let pos_after = h.position(enemy);
    assert!(
        pos_after.x.abs() < 30.0,
        "E 应把敌人拉到脚下（x≈0），实际 x={}",
        pos_after.x
    );
    h.finish();
}

/// 锥形内敌人应被眩晕 0.5s。
#[test]
fn sett_e_stuns_hit_enemies() {
    let mut h = build_headless("sett_e_stun");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    h.cast_skill(2, Vec2::new(800.0, 0.0)).advance(0.3);

    assert!(is_stunned(&h, enemy), "E 命中敌人应被眩晕 0.5s");
    h.finish();
}

/// 锥形外（超出范围或超出角度）的敌人不应被命中。
#[test]
fn sett_e_misses_outside_cone() {
    let mut h = build_headless("sett_e_miss");
    // 超出范围 490
    let far = h.add_enemy(Vec3::new(600.0, 0.0, 0.0));
    // 超出角度（半角 45°）：~56° 方向、距离 < 490
    let side = h.add_enemy(Vec3::new(200.0, 0.0, 300.0));
    let hp_far = h.health(far);
    let hp_side = h.health(side);

    h.cast_skill(2, Vec2::new(800.0, 0.0)).advance(0.3);

    assert!(
        (h.health(far) - hp_far).abs() < 0.5,
        "超出范围的敌人不应被 E 命中"
    );
    assert!(
        (h.health(side) - hp_side).abs() < 0.5,
        "锥形外的敌人不应被 E 命中"
    );
    h.finish();
}
