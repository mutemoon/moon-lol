#![cfg(test)]

//! 莫德凯撒被动 - 黑暗起兮 (Darkness Rise) 测试。

use bevy::math::{Vec2, Vec3};

use crate::mordekaiser::tests::*;

/// 普攻命中叠 1 层 Darkness。
#[test]
fn mordekaiser_passive_stacks_on_hit() {
    let mut h = build_headless("morde_passive_stacks");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    morde_hit(&mut h, enemy, 50.0);
    h.advance(0.1);
    assert_eq!(darkness_stacks(&h), Some(1), "首次命中应有 1 层");
}

/// 满 3 层激活，提供 3% 移速。
#[test]
fn mordekaiser_passive_activates_at_3_stacks() {
    let mut h = build_headless("morde_passive_activate");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    let speed_before = morde_speed(&h);
    for _ in 0..3 {
        morde_hit(&mut h, enemy, 10.0);
        h.advance(0.05);
    }
    assert!(darkness_active(&h), "满 3 层应激活");
    assert_eq!(darkness_stacks(&h), Some(3));
    let speed_after = morde_speed(&h);
    assert!(speed_after > speed_before, "激活应提升移速");
    let bonus = speed_after - speed_before;
    assert!(
        (bonus - speed_before * 0.03).abs() < 0.5,
        "移速加成约 3%（{}），实际 {}",
        speed_before * 0.03,
        bonus
    );
}

/// 激活期间普攻附带 40% AP 魔法伤害。
#[test]
fn mordekaiser_passive_auto_bonus() {
    let mut h = build_headless("morde_passive_auto_bonus");
    give_ap(&mut h, 100.0);
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    // 激活被动
    for _ in 0..3 {
        morde_hit(&mut h, enemy, 10.0);
        h.advance(0.05);
    }
    assert!(darkness_active(&h));
    // 第 4 次普攻：50 物理 + 40 魔法附伤（40% × 100 AP）
    let hp_before = h.health(enemy);
    morde_hit(&mut h, enemy, 50.0);
    h.advance(0.2);
    let dealt = hp_before - h.health(enemy);
    assert!(
        dealt >= 85.0,
        "普攻 + 40% AP 附伤应造成约 90 伤害，实际 {}",
        dealt
    );
}

/// 激活期间每 0.5 秒对半径内敌人造成 30% AP×层数 魔法伤害。
#[test]
fn mordekaiser_passive_dot() {
    let mut h = build_headless("morde_passive_dot");
    give_ap(&mut h, 100.0);
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    for _ in 0..3 {
        morde_hit(&mut h, enemy, 10.0);
        h.advance(0.05);
    }
    assert!(darkness_active(&h));
    let hp_before = h.health(enemy);
    // 推进超过一个 DoT 周期（0.5s）
    h.advance(0.6);
    let dealt = hp_before - h.health(enemy);
    // DoT = 0.3 × 100 AP × 3 层 = 90 / 周期
    assert!(dealt >= 80.0, "DoT 应造成伤害，实际 {}", dealt);
}

/// 脱战 4 秒后失效，还原移速。
#[test]
fn mordekaiser_passive_expires_out_of_combat() {
    let mut h = build_headless("morde_passive_expire");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    for _ in 0..3 {
        morde_hit(&mut h, enemy, 10.0);
        h.advance(0.05);
    }
    assert!(darkness_active(&h));
    let speed_active = morde_speed(&h);
    // 脱战 4 秒
    h.advance(4.2);
    assert!(!darkness_active(&h), "脱战 4s 应失效");
    assert_eq!(darkness_stacks(&h), Some(0));
    assert!(
        morde_speed(&h) < speed_active,
        "失效应还原移速 {} -> {}",
        speed_active,
        morde_speed(&h)
    );
    let _ = Vec2::ZERO;
}
