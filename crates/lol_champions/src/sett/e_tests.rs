#![cfg(test)]

//! Sett E（迎面痛击）集成测试（TDD）。
//!
//! E：双锥形检测 + 拉回 + 物理伤害。
//! 双侧均命中 → 全部眩晕；单侧命中 → 该侧减速。
//! Sett 基础 AD=60，1 级伤害 = 30+0.6×60 = 66。

use bevy::math::{Vec2, Vec3};

use super::tests::{build_headless, is_slowed, is_stunned};

/// 前方单个敌人被 E 拉回 + 伤害 + 减速。
#[test]
fn sett_e_pulls_and_damages_and_slows_single_side() {
    let mut h = build_headless("sett_e_pull_single");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    let hp_before = h.health(enemy);
    h.cast_skill(2, Vec2::new(800.0, 0.0)).advance(0.6);

    // 拉回：敌人应大幅移向 Sett（x 从 300 → 接近 0）
    let pos = h.position(enemy);
    assert!(
        pos.x < 50.0,
        "前方敌人应被拉回到 Sett 附近，实际 x = {:.1}",
        pos.x
    );

    // 伤害：应受 66 物理伤害（30 + 0.6×60）
    let dealt = hp_before - h.health(enemy);
    assert!(
        (dealt - 66.0).abs() < 0.5,
        "E 应造成 66 伤害，实际 = {dealt:.1}"
    );

    // 减速：单侧命中 → 减速，非眩晕
    assert!(is_slowed(&h, enemy), "单侧被 E 命中的敌人应减速");
    assert!(!is_stunned(&h, enemy), "单侧被 E 命中的敌人不应眩晕");
    h.finish();
}

/// 前后锥形均命中 → 全部眩晕。
#[test]
fn sett_e_dual_side_stuns_all() {
    let mut h = build_headless("sett_e_dual_stun");
    let front = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let back = h.add_enemy(Vec3::new(-300.0, 0.0, 0.0));

    h.cast_skill(2, Vec2::new(800.0, 0.0)).advance(0.3);

    assert!(is_stunned(&h, front), "前方敌人应被眩晕（双侧命中）");
    assert!(is_stunned(&h, back), "后方敌人应被眩晕（双侧命中）");
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
