#![cfg(test)]

use bevy::math::{Vec2, Vec3};

use super::tests::*;

/// Q1 中心区（敌人距施法者 < 200）：基础伤害 = -5 + 0.525*60 = 26.5（物理，护甲 0 全额）。
#[test]
fn q1_center_damage() {
    let mut h = build_headless("aatrox_q1_center");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    h.advance(0.1);

    let hp_before = h.health(enemy);
    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.1);
    let dealt = hp_before - h.health(enemy);

    assert!((dealt - 26.5).abs() < 1.5, "Q1 中心应为 26.5，实际 {dealt}");
}

/// Q1 边缘区（sweet spot，距离 ∈ [200, 300]）：1.7 倍伤害 + 击飞。
/// 26.5 * 1.7 = 45.05。
#[test]
fn q1_sweet_spot_knockup() {
    let mut h = build_headless("aatrox_q1_sweet");
    let enemy = h.add_enemy(Vec3::new(250.0, 0.0, 0.0));
    h.advance(0.1);

    let hp_before = h.health(enemy);
    h.cast_skill(0, Vec2::new(250.0, 0.0)).advance(0.1);
    let dealt = hp_before - h.health(enemy);

    assert!(
        (dealt - 45.05).abs() < 1.5,
        "Q1 边缘应为 45.05，实际 {dealt}"
    );
    assert!(is_knockup(&h, enemy), "Q1 边缘应附带击飞");
}

/// 三段 Q 每段 +25% 伤害：Q1=26.5、Q2=33.125、Q3=39.75。
#[test]
fn q_ramp_damage() {
    let mut h = build_headless("aatrox_q_ramp");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    h.advance(0.1);

    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.1);
    let hp_after_q1 = h.health(enemy);

    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.1);
    let hp_after_q2 = h.health(enemy);
    let q2 = hp_after_q1 - hp_after_q2;

    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.1);
    let q3 = hp_after_q2 - h.health(enemy);

    assert!((q2 - 33.125).abs() < 1.5, "Q2 应为 33.125，实际 {q2}");
    assert!((q3 - 39.75).abs() < 1.5, "Q3 应为 39.75，实际 {q3}");
}

/// 三段重施窗口：Q1->stage2、Q2->stage3、Q3->关闭并进入冷却。
#[test]
fn q_recast_window_progression() {
    let mut h = build_headless("aatrox_q_recast");
    h.advance(0.1);

    let q = h.skill_entity(0);

    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.1);
    assert_eq!(h.recast_window_stage(q), Some(2), "Q1 后应进入第 2 段重施");

    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.1);
    assert_eq!(h.recast_window_stage(q), Some(3), "Q2 后应进入第 3 段重施");

    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.1);
    assert!(!h.has_recast_window(q), "Q3 后应关闭重施窗口并进入冷却");
}
