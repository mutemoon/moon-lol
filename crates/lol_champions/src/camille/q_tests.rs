#![cfg(test)]

//! Camille Q（精准协议 / Precision Protocol）集成测试（TDD）。
//!
//! Q 重构为统一 on-hit 强化普攻：
//! - Q1：重置普攻 + 强化下次普攻（额外 = AD × TADRatio），开启重施窗口。
//! - Q2（重施）：重置普攻 + 强化下次普攻（额外 = AD × TADRatio × QEmpoweredAmp）。
//! 强化由通用 `BuffOnHitCounter` + `BuffOnHitBonusDamage` 承载，命中时由
//! `on_event_attack_end_consume_on_hit` 消费。

use bevy::math::{Vec2, Vec3};
use bevy::prelude::Entity;
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::Buffs;
use lol_core::buffs::on_hit::BuffOnHitBonusDamage;
use lol_core::damage::Damage;

use super::tests::build_headless;
use crate::test_utils::ChampionTestHarness;

/// 读取强化普攻额外伤害 buff（若存在）。
fn on_hit_bonus(h: &ChampionTestHarness) -> Option<BuffOnHitBonusDamage> {
    let buffs = h.app.world().get::<Buffs>(h.champion)?;
    for buff in buffs.iter() {
        if let Some(b) = h.app.world().get::<BuffOnHitBonusDamage>(*buff) {
            return Some(b.clone());
        }
    }
    None
}

/// 读取角色当前攻击力。
fn ad(h: &ChampionTestHarness) -> f32 {
    h.app
        .world()
        .get::<Damage>(h.champion)
        .map(|d| d.0)
        .unwrap_or(0.0)
}

/// 手动触发一次普攻命中（仅触发 on-hit，不造成基础 AA 伤害）。
fn attack_end(h: &mut ChampionTestHarness, target: Entity) {
    h.app.world_mut().trigger(EventAttackEnd {
        entity: h.champion,
        target,
    });
}

/// Q1 应赋予强化普攻 buff（ratio = TADRatio，1 级 = 0.15）。
#[test]
fn camille_q1_grants_on_hit_bonus() {
    let mut h = build_headless("camille_q1_bonus");
    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.2);

    let bonus = on_hit_bonus(&h).expect("Q1 后应有强化普攻额外伤害 buff");
    assert!(
        (bonus.ratio - 0.15).abs() < 1e-3,
        "Q1 ratio 应为 0.15，实际 {:.3}",
        bonus.ratio
    );
    assert!(bonus.flat.abs() < 1e-3, "Q1 flat 应为 0");
    h.finish();
}

/// Q1 强化应在普攻命中时造成额外物理伤害（= AD × 0.15），且仅此额外伤害（无基础 AA）。
#[test]
fn camille_q1_bonus_dealt_on_attack() {
    let mut h = build_headless("camille_q1_dealt");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.2);
    let hp_before = h.health(enemy);
    attack_end(&mut h, enemy);
    h.advance(0.1);

    let expected = ad(&h) * 0.15;
    let dealt = hp_before - h.health(enemy);
    assert!(
        (dealt - expected).abs() < 1.0,
        "Q1 强化应造成 ≈{:.1} 额外伤害（AD×0.15），实际 {:.1}",
        expected,
        dealt
    );
    h.finish();
}

/// Q2（重施）应将强化比例提升为 TADRatio × QEmpoweredAmp（0.15 × 2.0 = 0.3）。
#[test]
fn camille_q2_empowered_bonus() {
    let mut h = build_headless("camille_q2_empowered");
    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.2);
    // Q1 -> Q2 重施
    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.2);

    let bonus = on_hit_bonus(&h).expect("Q2 后应有强化普攻 buff");
    assert!(
        (bonus.ratio - 0.3).abs() < 1e-3,
        "Q2 ratio 应为 0.3（0.15×2.0），实际 {:.3}",
        bonus.ratio
    );
    h.finish();
}

/// 强化普攻在命中后应被消费（buff 移除）。
#[test]
fn camille_q_bonus_consumed_after_attack() {
    let mut h = build_headless("camille_q_consumed");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.2);
    assert!(on_hit_bonus(&h).is_some(), "命中前应存在强化 buff");

    attack_end(&mut h, enemy);
    h.advance(0.2);
    assert!(on_hit_bonus(&h).is_none(), "命中后强化 buff 应被消费移除");
    h.finish();
}
