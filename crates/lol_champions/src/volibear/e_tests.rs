#![cfg(test)]

//! Volibear E 集成测试（TDD）。
//!
//! E 为地面靶向延迟 AoE（落雷）：在**施法目标点**（非施法者位置）延迟后造成魔法伤害 + 减速。
//! 这是对 `AoEOrigin::CastPoint` 原语的验证：伤害以施法点为圆心，施法者位置不受影响。
//!
//! - 延迟 = castFrame 25/30 ≈ 0.833s
//! - 半径 = castRadius 325（从 ron 读取）
//! - 伤害 = spell ron 的 `calculated_damage`（魔法）
//! - 减速由 `on_volibear_damage_hit` observer 在伤害结算时施加（验证延迟伤害仍触发 observer）

use bevy::math::{Vec2, Vec3};
use lol_core::base::buff::Buffs;
use lol_core::buffs::cc_debuffs::DebuffSlow;

use super::tests::build_headless;
use crate::test_utils::*;

/// 延迟生效：castFrame 25/30≈0.833s 前无伤害，之后施法点敌人受伤。
#[test]
fn volibear_e_delayed_at_cast_point() {
    let mut h = build_headless("volibear_e_delayed");
    // 敌人就在施法点
    let enemy = h.add_enemy(Vec3::new(800.0, 0.0, 0.0));
    let hp_before = h.health(enemy);
    let mana_before = h.mana();

    h.cast_skill(2, Vec2::new(800.0, 0.0)).advance(0.5);
    assert!(
        (h.health(enemy) - hp_before).abs() < 0.01,
        "延迟结束前不应造成伤害"
    );

    h.advance(0.5); // 总 1.0s > 0.833
    let dealt = hp_before - h.health(enemy);
    assert!(dealt > 0.0, "延迟结束后施法点敌人应受到魔法伤害");
    assert!(
        !h.can_cast(2),
        "E 施放后应进入冷却"
    );
    assert!(h.mana() < mana_before, "E 施放应消耗法力");
    h.finish();
}

/// CastPoint 验证：施法者位置的敌人不受伤害（原点是施法点，不是施法者）。
#[test]
fn volibear_e_caster_location_unharmed() {
    let mut h = build_headless("volibear_e_caster_loc");
    // 敌人在施法者位置（原点），E 打向 (800,0)
    let enemy = h.add_enemy(Vec3::new(0.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    h.cast_skill(2, Vec2::new(800.0, 0.0)).advance(1.0);

    assert!(
        (h.health(enemy) - hp_before).abs() < 0.01,
        "施法者位置敌人不应受伤（CastPoint 原点为施法点 800,0，距施法者 800 > 325）"
    );
    h.finish();
}

/// 半径边界：施法点 300 内命中，400 外不命中（castRadius=325）。
#[test]
fn volibear_e_radius_boundary() {
    let mut h = build_headless("volibear_e_radius");
    let inner = h.add_enemy(Vec3::new(800.0, 0.0, 300.0));
    let outer = h.add_enemy(Vec3::new(800.0, 0.0, 400.0));
    let hp_inner = h.health(inner);
    let hp_outer = h.health(outer);

    h.cast_skill(2, Vec2::new(800.0, 0.0)).advance(1.0);

    assert!(h.health(inner) < hp_inner, "半径内（300<325）应受伤");
    assert!(
        (h.health(outer) - hp_outer).abs() < 0.01,
        "半径外（400>325）不应受伤"
    );
    h.finish();
}

/// 延迟伤害仍触发命中 observer（减速）。
#[test]
fn volibear_e_slow_applied() {
    let mut h = build_headless("volibear_e_slow");
    let enemy = h.add_enemy(Vec3::new(800.0, 0.0, 0.0));

    h.cast_skill(2, Vec2::new(800.0, 0.0)).advance(1.0);

    // DebuffSlow 经 BuffOf 关联挂在 buff 子实体上，需遍历敌人的 Buffs 查找
    let mut has_slow = false;
    if let Some(buffs) = h.app.world().get::<Buffs>(enemy) {
        for buff in buffs.iter() {
            if h.app.world().get::<DebuffSlow>(*buff).is_some() {
                has_slow = true;
                break;
            }
        }
    }
    assert!(has_slow, "延迟伤害应触发 on_volibear_damage_hit 施加减速");
    h.finish();
}
