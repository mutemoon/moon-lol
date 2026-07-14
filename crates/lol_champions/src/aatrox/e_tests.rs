#![cfg(test)]

use bevy::math::Vec2;

use super::tests::*;

/// E 向指针方向突进，到达落点（最大 300）。
#[test]
fn e_dashes_toward_point() {
    let mut h = build_headless("aatrox_e_dash");
    h.advance(0.1);

    h.cast_skill(2, Vec2::new(300.0, 0.0)).advance(0.5);
    let x = h.position(h.champion).x;

    assert!(x > 290.0, "E 应突进到 (300,0) 附近，实际 x={x}");
}

/// E 超过最大距离时钳制到 300。
#[test]
fn e_respects_max_range() {
    let mut h = build_headless("aatrox_e_maxrange");
    h.advance(0.1);

    h.cast_skill(2, Vec2::new(1000.0, 0.0)).advance(0.5);
    let x = h.position(h.champion).x;

    assert!(
        x <= 305.0 && x > 290.0,
        "E 应钳制到最大距离 300，实际 x={x}"
    );
}
