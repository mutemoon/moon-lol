#![cfg(test)]

//! Sett 被动（沙场战心）集成测试（TDD）。
//!
//! 被动：左右拳交替，右拳（偶数次普攻）附带 0.55×AD 额外物理伤害。
//! Sett 基础 AD=60，故右拳附伤 = 33。

use bevy::math::Vec3;

use super::tests::{attack_end, build_headless};

/// 左拳（第 1 次）不附带额外伤害。
#[test]
fn sett_passive_left_punch_no_bonus() {
    let mut h = build_headless("sett_passive_left");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    attack_end(&mut h, enemy);
    h.advance(0.1);

    let dealt = hp_before - h.health(enemy);
    assert!(dealt.abs() < 0.5, "左拳不应附带额外伤害，实际 {}", dealt);
    h.finish();
}

/// 右拳（第 2 次）附带 0.55×AD = 33 物理伤害。
#[test]
fn sett_passive_right_punch_bonus() {
    let mut h = build_headless("sett_passive_right");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    attack_end(&mut h, enemy); // 左拳
    h.advance(0.1);
    attack_end(&mut h, enemy); // 右拳
    h.advance(0.1);

    let dealt = hp_before - h.health(enemy);
    assert!(
        (dealt - 33.0).abs() < 1.0,
        "右拳应附带 0.55*AD=33 物理伤害，实际 {}",
        dealt
    );
    h.finish();
}

/// 4 次普攻：第 2、4 次为右拳，总附伤 66。
#[test]
fn sett_passive_alternates_over_four() {
    let mut h = build_headless("sett_passive_four");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    for _ in 0..4 {
        attack_end(&mut h, enemy);
        h.advance(0.1);
    }

    let dealt = hp_before - h.health(enemy);
    assert!(
        (dealt - 66.0).abs() < 1.5,
        "4 次普攻应附伤 66（2 次右拳 ×33），实际 {}",
        dealt
    );
    h.finish();
}
