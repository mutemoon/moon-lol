#![cfg(test)]

//! 莫德凯撒 R - 轮回绝境 测试。

use bevy::math::{Vec2, Vec3};
use lol_core::damage::{Armor, Damage};
use lol_core::life::Health;

use crate::mordekaiser::buffs::MordekaiserStatSteal;
use crate::mordekaiser::tests::*;

/// R 命中附近最近敌人，施加死亡领域。
#[test]
fn mordekaiser_r_applies_realm() {
    let mut h = build_headless("morde_r_realm");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.3);
    assert!(has_realm(&h), "R 应开启死亡领域");
    assert_eq!(realm_target(&h), Some(enemy), "领域目标应为该敌人");
}

/// 领域持续 7 秒后结束。
#[test]
fn mordekaiser_r_realm_expires() {
    let mut h = build_headless("morde_r_expire");
    let _enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.3);
    assert!(has_realm(&h));
    h.advance(7.2);
    assert!(!has_realm(&h), "7s 后领域应结束");
}

/// 领域内击杀目标窃取 10% 属性。
#[test]
fn mordekaiser_r_stat_steal_on_kill() {
    let mut h = build_headless("morde_r_steal");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    // 给予敌人属性
    h.app.world_mut().entity_mut(enemy).insert(Damage(200.0));
    h.app.world_mut().entity_mut(enemy).insert(Armor(100.0));

    let ad_before = morde_ad(&h);
    let armor_before = morde_armor(&h);
    let maxhp_before = morde_max_hp(&h);

    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.3);
    assert!(has_realm(&h));

    // 击杀目标
    h.app
        .world_mut()
        .entity_mut(enemy)
        .get_mut::<Health>()
        .unwrap()
        .value = 0.0;
    h.advance(0.3);

    let ad_after = morde_ad(&h);
    let armor_after = morde_armor(&h);
    let maxhp_after = morde_max_hp(&h);
    assert!(
        ad_after > ad_before,
        "应窃取 AD: {} -> {}",
        ad_before,
        ad_after
    );
    assert!(
        armor_after > armor_before,
        "应窃取护甲: {} -> {}",
        armor_before,
        armor_after
    );
    assert!(
        maxhp_after > maxhp_before,
        "应窃取生命: {} -> {}",
        maxhp_before,
        maxhp_after
    );

    let steal: &MordekaiserStatSteal = &stat_steal(&h).expect("应有窃取记录");
    assert!(
        (steal.ad - 20.0).abs() < 1.0,
        "窃取 AD 应为 20（10% × 200），实际 {}",
        steal.ad
    );
    assert!(
        (steal.armor - 10.0).abs() < 1.0,
        "窃取护甲应为 10（10% × 100），实际 {}",
        steal.armor
    );

    let _ = Vec2::ZERO;
    let _ = Vec3::ZERO;
}
