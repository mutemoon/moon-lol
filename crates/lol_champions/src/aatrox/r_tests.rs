#![cfg(test)]

use bevy::math::Vec2;

use super::tests::*;

/// R 提升 AD：60 * 0.1 = 6 -> 66。
#[test]
fn r_grants_bonus_ad() {
    let mut h = build_headless("aatrox_r_ad");
    h.advance(0.1);

    let ad_before = ad(&h);
    h.cast_skill(3, Vec2::new(0.0, 0.0)).advance(0.6);
    let ad_after = ad(&h);

    assert!(
        (ad_before - 60.0).abs() < 0.5,
        "基础 AD 应为 60，实际 {ad_before}"
    );
    assert!(
        (ad_after - 66.0).abs() < 0.5,
        "R 期间 AD 应为 66，实际 {ad_after}"
    );
}

/// R 期间获得移速增益。
#[test]
fn r_grants_movement_speed() {
    let mut h = build_headless("aatrox_r_ms");
    h.advance(0.1);

    h.cast_skill(3, Vec2::new(0.0, 0.0)).advance(0.6);
    assert!(has_movespeed_buff(&h, h.champion), "R 应附加移速增益");
}

/// R 持续 10s，到期后移除额外 AD。
#[test]
fn r_duration_expires() {
    let mut h = build_headless("aatrox_r_expire");
    h.advance(0.1);

    h.cast_skill(3, Vec2::new(0.0, 0.0)).advance(0.6);
    assert!((ad(&h) - 66.0).abs() < 0.5, "R 期间 AD=66");

    h.advance(10.0);
    assert!(
        (ad(&h) - 60.0).abs() < 0.5,
        "R 到期后 AD 应恢复 60，实际 {}",
        ad(&h)
    );
}
