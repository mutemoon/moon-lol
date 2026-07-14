#![cfg(test)]

//! 莫德凯撒 W - 不坏 metamorphosis（不可破坏）测试。

use bevy::math::{Vec2, Vec3};
use lol_core::damage::Armor;

use crate::mordekaiser::tests::*;

/// W 首次施放产生 5% 最大生命护盾（无储存）。
#[test]
fn mordekaiser_w_creates_shield() {
    let mut h = build_headless("morde_w_shield");
    let max_hp = morde_max_hp(&h);
    h.cast_skill(1, Vec2::ZERO).advance(0.2);
    let shield = h.shield_value().expect("W 应产生护盾");
    let expected = max_hp * 0.05;
    assert!(
        (shield - expected).abs() < 1.0,
        "护盾应为 5% 最大生命 {:.1}，实际 {:.1}",
        expected,
        shield
    );
}

/// 受伤时储存 7.5% 已损伤害（护甲清零以得整数）。
#[test]
fn mordekaiser_w_passive_storage() {
    let mut h = build_headless("morde_w_storage");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    // 护甲清零，final_damage = 1000
    h.app.world_mut().entity_mut(h.champion).insert(Armor(0.0));
    morde_take_damage(&mut h, enemy, 1000.0);
    h.advance(0.2);
    let stored = w_storage(&h).expect("受伤应产生 W 储存");
    assert!(
        (stored - 75.0).abs() < 1.0,
        "储存应为 75（7.5% × 1000），实际 {}",
        stored
    );
}

/// W 储存并入护盾，重施放治疗自身并消耗护盾。
#[test]
fn mordekaiser_w_recast_heals() {
    let mut h = build_headless("morde_w_recast");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    h.app.world_mut().entity_mut(h.champion).insert(Armor(0.0));
    // 受伤降低血量并产生储存
    morde_take_damage(&mut h, enemy, 300.0);
    h.advance(0.2);
    let hp_before_w = h.health(h.champion);
    assert!(w_storage(&h).unwrap() > 0.0);
    // 首次 W：开盾（不改变血量）
    h.cast_skill(1, Vec2::ZERO).advance(0.2);
    let hp_after_first_w = h.health(h.champion);
    assert!(
        (hp_after_first_w - hp_before_w).abs() < 1.0,
        "开盾不应改变血量"
    );
    assert!(h.shield_value().is_some(), "护盾应存在");
    // 重施 W：治疗并消耗护盾
    h.cast_skill(1, Vec2::ZERO).advance(0.3);
    let hp_after_recast = h.health(h.champion);
    assert!(
        hp_after_recast > hp_after_first_w,
        "重施应治疗自身 {} -> {}",
        hp_after_first_w,
        hp_after_recast
    );
    assert!(
        h.shield_value().map(|s| s < 1.0).unwrap_or(true),
        "重施应消耗护盾"
    );
}

/// 护盾在 1 秒后开始衰减。
#[test]
fn mordekaiser_w_shield_decays() {
    let mut h = build_headless("morde_w_decay");
    h.cast_skill(1, Vec2::ZERO).advance(0.2);
    let shield_initial = h.shield_value().expect("W 护盾应存在");
    // 推进到衰减阶段（> 1s）
    h.advance(2.0);
    let shield_later = h.shield_value();
    assert!(
        shield_later.map(|s| s < shield_initial).unwrap_or(true),
        "衰减后护盾应减少，初始 {:.1} 之后 {:?}",
        shield_initial,
        shield_later
    );
}
