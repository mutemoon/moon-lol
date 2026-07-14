#![cfg(test)]

//! Camille 被动（自适应防御 / Adaptive Defenses）集成测试（TDD）。
//!
//! 被动：对敌方英雄的普攻命中后获得护盾（6% 最大生命值，持续 2s），
//! 护盾用通用 `BuffShieldWhite` 承载（抵挡全类型伤害）。

use bevy::math::{Vec2, Vec3};
use lol_core::attack::EventAttackEnd;

use super::tests::build_headless;
use crate::test_utils::ChampionTestHarness;

/// 辅助：手动触发一次普攻命中（只触发 on-hit，不造成基础 AA 伤害）。
fn attack_end(h: &mut ChampionTestHarness, target: bevy::prelude::Entity) {
    h.app.world_mut().trigger(EventAttackEnd {
        entity: h.champion,
        target,
    });
}

/// 普攻命中敌方英雄应获得护盾（6% 最大生命值）。
#[test]
fn camille_passive_grants_shield_on_champion_hit() {
    let mut h = build_headless("camille_passive_shield");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    attack_end(&mut h, enemy);
    h.advance(0.1);

    let max_hp = h
        .app
        .world()
        .get::<lol_core::life::Health>(h.champion)
        .unwrap()
        .max;
    let shield = h.shield_value().expect("被动应产生护盾");
    assert!(
        (shield - max_hp * 0.06).abs() < 0.5,
        "护盾值应为 6% 最大生命值 ({:.1})，实际 {:.1}",
        max_hp * 0.06,
        shield
    );
    h.finish();
}

/// 护盾 2s 后应自动消失。
#[test]
fn camille_passive_shield_expires() {
    let mut h = build_headless("camille_passive_expire");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    attack_end(&mut h, enemy);
    h.advance(0.1);
    assert!(h.shield_value().is_some(), "命中后应存在护盾");

    h.advance(2.2); // 总计 > 2s
    assert!(h.shield_value().is_none(), "2s 后护盾应消失");
    h.finish();
}

/// 护盾应抵挡伤害：在护盾存在时受到小于护盾值的伤害，生命值不应下降。
#[test]
fn camille_passive_shield_absorbs_damage() {
    let mut h = build_headless("camille_passive_absorb");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    attack_end(&mut h, enemy);
    h.advance(0.1);

    let shield = h.shield_value().unwrap();
    let hp_before = h.health(h.champion);
    // 受到小于护盾值的伤害（敌人作为 source）
    h.apply_damage(enemy, shield * 0.5);
    h.advance(0.1);

    assert!(
        (h.health(h.champion) - hp_before).abs() < 0.5,
        "护盾应完全吸收小于其值的伤害，生命值应不变"
    );
    h.finish();
}

/// 重新命中应刷新护盾（覆盖旧计时器，重新获得满额护盾）。
#[test]
fn camille_passive_refresh_on_rehit() {
    let mut h = build_headless("camille_passive_refresh");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    attack_end(&mut h, enemy);
    h.advance(1.5); // 已经过去 1.5s（接近过期）

    attack_end(&mut h, enemy); // 重新命中刷新
    h.advance(1.0); // 距首次命中 2.5s，但距刷新 1.0s

    assert!(
        h.shield_value().is_some(),
        "重新命中应刷新护盾计时，2.5s 后（距刷新 1s）护盾应仍存在"
    );
    h.finish();
}

// 避免 Vec2 未使用告警（保留以便后续扩展测试使用指针）
#[allow(dead_code)]
fn _unused(_: Vec2) {}
