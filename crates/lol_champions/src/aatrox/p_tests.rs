#![cfg(test)]

use bevy::math::Vec3;

use super::tests::*;

/// 被动就绪时，下次普攻附带目标最大生命值 15% 的额外魔法伤害。
/// 敌人 6000 HP -> 额外 900（无魔抗，全额）。
#[test]
fn p_empowers_attack_bonus_damage() {
    let mut h = build_headless("aatrox_p_empowers");
    let enemy = h.add_enemy(Vec3::new(150.0, 0.0, 0.0));
    h.advance(0.1);

    let hp_before = h.health(enemy);
    attack_end(&mut h, enemy);
    h.advance(0.1);

    let dealt = hp_before - h.health(enemy);
    assert!(
        (dealt - 900.0).abs() < 5.0,
        "被动应造成 6000*0.15=900 额外伤害，实际 {dealt}"
    );
}

/// 触发一次后进入冷却，第二次普攻不再附带额外伤害。
#[test]
fn p_goes_on_cooldown() {
    let mut h = build_headless("aatrox_p_cooldown");
    let enemy = h.add_enemy(Vec3::new(150.0, 0.0, 0.0));
    h.advance(0.1);

    attack_end(&mut h, enemy);
    h.advance(0.1);
    let hp_after_first = h.health(enemy);

    attack_end(&mut h, enemy);
    h.advance(0.1);
    let dealt_second = hp_after_first - h.health(enemy);

    assert!(
        dealt_second < 1.0,
        "冷却中的被动不应再造成额外伤害，实际二次伤害 {dealt_second}"
    );
}

/// 被动触发时治疗自身（已损生命值才有效；满血时无效）。
#[test]
fn p_heals_self() {
    let mut h = build_headless("aatrox_p_heal");
    let enemy = h.add_enemy(Vec3::new(150.0, 0.0, 0.0));
    h.advance(0.1);

    // 先受伤制造血量缺口。
    h.apply_damage(enemy, 400.0).advance(0.1);
    let hp_low = h.health(h.champion);
    assert!(hp_low < max_health(&h, h.champion), "应先受伤");

    attack_end(&mut h, enemy);
    h.advance(0.1);
    let hp_after = h.health(h.champion);

    assert!(hp_after > hp_low, "被动应治疗自身：{hp_low} -> {hp_after}");
    assert!(
        hp_after <= max_health(&h, h.champion) + 0.01,
        "治疗不应超过最大生命值"
    );
}
