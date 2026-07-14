#![cfg(test)]

//! Sett W 集成测试（TDD）。
//!
//! W 向前挥出锥形：**中心窄扇形（30°）为真实伤害，两侧（全 75° 扇形排除中心）为物理伤害**。
//! 这是对 `ActionDamageEffect::exclude` 空间分区原语的验证：
//! - 中心只被真实 effect 命中（exclude 防止被物理 effect 重复命中）
//! - 两侧只被物理 effect 命中（不在中心扇形内）
//!
//! 伤害值来自 spell ron 的 `damage_calc`（= BaseDamage，1 级 60）。
//! 通过给敌人不同护甲区分伤害类型：真实无视护甲，物理按 100/(100+armor) 减免。

use bevy::math::{Vec2, Vec3};
use lol_core::damage::Armor;

use super::tests::{build_headless, grit_value};

/// 中心敌人带高护甲仍受满额伤害（真实无视护甲），证明中心为真实伤害。
#[test]
fn sett_w_center_true_ignores_armor() {
    let mut h = build_headless("sett_w_center_true");
    // 正前方 200、0°（在 30° 中心扇形内），给 200 护甲
    let center = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    h.app.world_mut().entity_mut(center).insert(Armor(200.0));
    let hp_before = h.health(center);

    h.cast_skill(1, Vec2::new(800.0, 0.0)).advance(0.3);

    // 真实伤害无视护甲：damage_calc(60) 全额。若误判为物理，则 60*100/300=20。
    let dealt = hp_before - h.health(center);
    assert!(
        dealt > 50.0,
        "中心应为真实伤害（无视护甲，全额 ~60），实际造成 {}",
        dealt
    );
    h.finish();
}

/// 两侧敌人受物理伤害，被护甲减免到 ~20（60*100/300）。
#[test]
fn sett_w_edge_physical_reduced_by_armor() {
    let mut h = build_headless("sett_w_edge_physical");
    // 距离 ~200、相对 +X 约 20°（在 75° 全扇形内、30° 中心外）
    let edge = h.add_enemy(Vec3::new(188.0, 0.0, 68.0));
    h.app.world_mut().entity_mut(edge).insert(Armor(200.0));
    let hp_before = h.health(edge);

    h.cast_skill(1, Vec2::new(800.0, 0.0)).advance(0.3);

    let dealt = hp_before - h.health(edge);
    assert!(
        (dealt - 20.0).abs() < 1.0,
        "两侧应为物理伤害（被 200 护甲减免到 ~20），实际造成 {}",
        dealt
    );
    h.finish();
}

/// 中心敌人只受一次伤害（exclude 防止被物理 effect 重复命中，否则会 120）。
#[test]
fn sett_w_center_single_hit() {
    let mut h = build_headless("sett_w_center_single");
    let center = h.add_enemy(Vec3::new(200.0, 0.0, 0.0)); // 无护甲
    let hp_before = h.health(center);

    h.cast_skill(1, Vec2::new(800.0, 0.0)).advance(0.3);

    // damage_calc(60)；若 exclude 失效，中心会被物理+真实双命中 = 120
    let dealt = hp_before - h.health(center);
    assert!(
        (dealt - 60.0).abs() < 1.0,
        "中心应只受一次 60 伤害（exclude 生效），实际 {}",
        dealt
    );
    h.finish();
}

/// 延迟生效：castFrame 5.265/30≈0.176s 前无伤害，之后有伤害。
#[test]
fn sett_w_delayed_damage() {
    let mut h = build_headless("sett_w_delayed");
    let center = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let hp_before = h.health(center);

    h.cast_skill(1, Vec2::new(800.0, 0.0)).advance(0.1);
    assert!(
        (h.health(center) - hp_before).abs() < 0.01,
        "延迟结束前不应造成伤害"
    );

    h.advance(0.3);
    assert!(h.health(center) < hp_before, "延迟结束后中心敌人应受到伤害");
    h.finish();
}

/// 扇形外敌人（超出半径或超出角度）不受伤害。
#[test]
fn sett_w_outside_sector_unharmed() {
    let mut h = build_headless("sett_w_outside");
    // 超出半径 350
    let far = h.add_enemy(Vec3::new(400.0, 0.0, 0.0));
    // 超出角度 75°（半角 37.5°）：~50° 方向、距离 < 350
    let side = h.add_enemy(Vec3::new(150.0, 0.0, 180.0));
    let hp_far = h.health(far);
    let hp_side = h.health(side);

    h.cast_skill(1, Vec2::new(800.0, 0.0)).advance(0.3);

    assert!((h.health(far) - hp_far).abs() < 0.01, "超出半径不受伤害");
    assert!(
        (h.health(side) - hp_side).abs() < 0.01,
        "超出扇形角度不受伤害"
    );
    h.finish();
}

/// W 主动应把已储存的"灰心"全额转化为白盾，并清零灰心。
#[test]
fn sett_w_shield_from_grit() {
    let mut h = build_headless("sett_w_grit_shield");
    // 先受伤累积灰心（受 200 物理伤害，被 33 护甲减免后 ~150）
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    h.apply_damage(enemy, 200.0);
    h.advance(0.1);
    let grit = grit_value(&h);
    assert!(
        grit > 100.0,
        "受伤 200 应储存可观灰心（护甲减免后 ~150），实际 {}",
        grit
    );

    // W 主动：灰心 -> 白盾
    h.cast_skill(1, Vec2::new(800.0, 0.0)).advance(0.1);
    let shield = h.shield_value().expect("W 应把灰心转化为白盾");
    assert!(
        (shield - grit).abs() < 1.0,
        "白盾应等于释放前的灰心 {:.1}，实际 {:.1}",
        grit,
        shield
    );
    assert!(grit_value(&h).abs() < 0.5, "W 释放后灰心应清零");
    h.finish();
}

/// 灰心脱战 4 秒后应衰减清零。
#[test]
fn sett_grit_decays_out_of_combat() {
    let mut h = build_headless("sett_grit_decay");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    h.apply_damage(enemy, 200.0);
    h.advance(0.1);
    assert!(grit_value(&h) > 100.0, "受伤后应有灰心");

    h.advance(4.2); // 脱战 4s 后衰减
    assert!(
        grit_value(&h).abs() < 0.5,
        "脱战 4s 后灰心应清零，实际 {}",
        grit_value(&h)
    );
    h.finish();
}
