#![cfg(test)]

//! Darius Q 集成测试（TDD - 延迟迁移）。
//!
//! Q 大杀四方为前摇延迟的双形 AoE：以**施法者**为圆心的全向劈砍，
//! castFrame 7.5/30 = 0.25s 前摇后造成物理伤害。这是对 `AoEOrigin::Caster`
//! + 双 `ActionDamageEffect`（Circle 内圈 + Annular 外圈）原语的组合验证：
//!
//! - 原点 = Caster（劈砍以施法者为中心，全向）
//! - 延迟 = castFrame 7.5 → 0.25s（前摇）
//! - 形状 = [Circle{150} 内圈, Annular{150,350} 外圈]（双形空间划分）
//! - 伤害 = spell ron 的 `blade_damage`（物理）
//! - 出血由 `on_darius_damage_hit` observer 在伤害结算时施加

use bevy::math::{Vec2, Vec3};
use lol_core::base::buff::Buffs;

use super::tests::{build_headless, give_mana};
use crate::darius::buffs::BuffDariusBleed;

/// 前摇 0.25s：0.2s 前无伤害，0.3s 后命中。
#[test]
fn darius_q_delayed_windup() {
    let mut h = build_headless("darius_q_delayed");
    give_mana(&mut h);
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let hp_before = h.health(enemy);
    let mana_before = h.mana();

    h.cast_skill(0, Vec2::new(200.0, 0.0)).advance(0.2);
    assert!(
        (h.health(enemy) - hp_before).abs() < 0.01,
        "castFrame 0.25s 前摇结束前不应造成伤害"
    );

    h.advance(0.2); // 总 0.4s > 0.25
    assert!(
        h.health(enemy) < hp_before,
        "前摇结束后外圈敌人应受到物理伤害"
    );
    assert!(h.mana() < mana_before, "Q 施放应消耗法力");
    h.finish();
}

/// 内圈命中：150 内（100）受内圈 Circle 伤害。
#[test]
fn darius_q_inner_blade_hits() {
    let mut h = build_headless("darius_q_inner");
    give_mana(&mut h);
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    let hp_before = h.health(enemy);
    let mana_before = h.mana();

    h.cast_skill(0, Vec2::new(200.0, 0.0)).advance(0.4);

    assert!(
        h.health(enemy) < hp_before,
        "内圈（150 内）应受 blade_damage 伤害"
    );
    assert!(!h.can_cast(0), "Q 施放后应进入冷却");
    assert!(h.mana() < mana_before, "Q 施放应消耗法力");
    h.finish();
}

/// 外圈边界：150-350 内命中，350 外不命中。
#[test]
fn darius_q_outer_blade_boundary() {
    let mut h = build_headless("darius_q_outer_boundary");
    give_mana(&mut h);
    let inside = h.add_enemy(Vec3::new(300.0, 0.0, 0.0)); // 150-350 内
    let outside = h.add_enemy(Vec3::new(400.0, 0.0, 0.0)); // 350 外
    let hp_in = h.health(inside);
    let hp_out = h.health(outside);
    let mana_before = h.mana();

    h.cast_skill(0, Vec2::new(300.0, 0.0)).advance(0.4);

    assert!(h.health(inside) < hp_in, "外圈（150-350）应受伤");
    assert!(
        (h.health(outside) - hp_out).abs() < 0.01,
        "外圈之外（>350）不应受伤"
    );
    assert!(!h.can_cast(0), "Q 施放后应进入冷却");
    assert!(h.mana() < mana_before, "Q 施放应消耗法力");
    h.finish();
}

/// 延迟伤害仍触发命中 observer（出血）。
#[test]
fn darius_q_delayed_hemorrhage() {
    let mut h = build_headless("darius_q_delayed_hemo");
    give_mana(&mut h);
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let mana_before = h.mana();

    h.cast_skill(0, Vec2::new(200.0, 0.0)).advance(0.4);

    let mut found_bleed = false;
    if let Some(buffs) = h.app.world().get::<Buffs>(enemy) {
        for buff in buffs.iter() {
            if h.app.world().get::<BuffDariusBleed>(*buff).is_some() {
                found_bleed = true;
                break;
            }
        }
    }
    assert!(found_bleed, "延迟伤害应触发 on_darius_damage_hit 叠出血");
    assert!(!h.can_cast(0), "Q 施放后应进入冷却");
    assert!(h.mana() < mana_before, "Q 施放应消耗法力");
    h.finish();
}
