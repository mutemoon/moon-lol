#![cfg(test)]

//! Volibear W（疯狂撕咬）集成测试。
//!
//! W1：咬最近敌人，造成 total_damage 物理伤害并标记 8s。
//! W2（重施）：对已标记目标造成 1.5x 伤害并自我治疗（BaseHeal + HealPercent*已损生命）。
//! total_damage @1级 = base(-20) + 1.1*AD + 0.06*maxHP = -20 + 71.5 + 39 = 90.5。

use bevy::math::{Vec2, Vec3};

use super::tests::{build_headless, has_mark};

/// W1 应对最近敌人造成 90.5 物理伤害并标记。
/// W1 开启 2s 重施窗口，窗口过期后进入冷却。
#[test]
fn volibear_w1_damages_and_marks() {
    let mut h = build_headless("volibear_w1_mark");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let hp_before = h.health(enemy);
    let mana_before = h.mana();

    h.cast_skill(1, Vec2::new(200.0, 0.0)).advance(0.1);

    let dealt = hp_before - h.health(enemy);
    assert!(
        (dealt - 90.5).abs() < 1.0,
        "W1 应造成 90.5 物理伤害，实际 {dealt}"
    );
    assert!(has_mark(&h, enemy), "W1 应标记目标 8s");
    assert!(h.mana() < mana_before, "W 施放应消耗法力");

    // W1 开启 2s 重施窗口，窗口过期后才进入 5s 冷却
    h.advance(2.5);
    assert!(!h.can_cast(1), "W 重施窗口过期后应进入冷却");
    h.finish();
}

/// W2 对已标记目标应造成 1.5x=135.75 伤害。
#[test]
fn volibear_w2_bonus_damage_on_marked() {
    let mut h = build_headless("volibear_w2_bonus");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));

    // W1 标记
    h.cast_skill(1, Vec2::new(200.0, 0.0)).advance(0.1);
    let hp_after_w1 = h.health(enemy);

    // W2 引爆
    h.cast_skill(1, Vec2::new(200.0, 0.0)).advance(0.1);
    let dealt_w2 = hp_after_w1 - h.health(enemy);

    assert!(
        (dealt_w2 - 135.75).abs() < 1.5,
        "W2 对已标记目标应造成 1.5x=135.75 伤害，实际 {dealt_w2}"
    );
    h.finish();
}

/// W2 引爆标记应治疗沃利贝尔（已损生命越多治疗越高）。
#[test]
fn volibear_w2_heals_on_marked() {
    let mut h = build_headless("volibear_w2_heal");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let champion = h.champion;

    // 先受伤降低生命，制造已损生命
    h.apply_damage(enemy, 300.0).advance(0.1);
    // W1 标记
    h.cast_skill(1, Vec2::new(200.0, 0.0)).advance(0.1);
    let hp_before_w2 = h.health(champion);

    // W2 引爆 + 治疗
    h.cast_skill(1, Vec2::new(200.0, 0.0)).advance(0.1);
    let hp_after_w2 = h.health(champion);

    assert!(
        hp_after_w2 - hp_before_w2 > 10.0,
        "W2 应治疗沃利贝尔（BaseHeal+5%已损生命≈16），治疗前 {hp_before_w2} 治疗后 {hp_after_w2}"
    );
    h.finish();
}
