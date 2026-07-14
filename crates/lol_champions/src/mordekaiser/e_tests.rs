#![cfg(test)]

//! 莫德凯撒 E - 断魂一曳 测试。

use bevy::math::{Vec2, Vec3};

use crate::mordekaiser::tests::*;

/// E 命中锥形内敌人：造成魔法伤害并拉向自身。
#[test]
fn mordekaiser_e_pulls_and_damages() {
    let mut h = build_headless("morde_e_pull");
    let enemy = h.add_enemy(Vec3::new(400.0, 0.0, 0.0));
    let hp_before = h.health(enemy);
    let pos_before = h.position(enemy);
    h.cast_skill(2, Vec2::new(400.0, 0.0)).advance(0.5);
    assert!(h.health(enemy) < hp_before, "E 应造成魔法伤害");
    let pos_after = h.position(enemy);
    assert!(
        pos_after.x < pos_before.x,
        "E 应将敌人拉近，{} -> {}",
        pos_before.x,
        pos_after.x
    );
}

/// E 伤害 = 基础 + 40% AP。
#[test]
fn mordekaiser_e_ap_scaling() {
    let mut h = build_headless("morde_e_ap");
    give_ap(&mut h, 100.0);
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let hp_before = h.health(enemy);
    h.cast_skill(2, Vec2::new(300.0, 0.0)).advance(0.5);
    let dealt = hp_before - h.health(enemy);
    // 基础 45 + 40% × 100 = 85
    assert!(
        (dealt - 85.0).abs() < 2.0,
        "E 应造成 85 伤害（45 + 40），实际 {}",
        dealt
    );
}
