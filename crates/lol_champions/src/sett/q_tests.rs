#![cfg(test)]

//! Sett Q（屈人之威）集成测试（TDD）。
//!
//! Q：强化下 2 次普攻，每次附带目标最大生命百分比物理伤害（1 级 3%）+ 30% 移速 4s。
//! 敌人最大生命 6000，故 1 级附伤 = 180。

use bevy::math::{Vec2, Vec3};

use super::tests::{attack_end, build_headless, level_skill};

/// Q 首攻应附带 3% 目标最大生命 = 180 物理伤害。
#[test]
fn sett_q_first_attack_max_hp_percent() {
    let mut h = build_headless("sett_q_first");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    h.cast_skill(0, Vec2::new(800.0, 0.0)).advance(0.1);
    attack_end(&mut h, enemy);
    h.advance(0.1);

    let dealt = hp_before - h.health(enemy);
    // 0.03 * 6000 = 180；首攻为左拳，无被动附伤
    assert!(
        (dealt - 180.0).abs() < 1.0,
        "Q 首攻应附带 3% 目标最大生命=180，实际 {}",
        dealt
    );
    h.finish();
}

/// Q 强化 2 次后耗尽，第 3 次普攻不再附带 Q 伤害。
#[test]
fn sett_q_expires_after_two_attacks() {
    let mut h = build_headless("sett_q_expires");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));

    h.cast_skill(0, Vec2::new(800.0, 0.0)).advance(0.1);
    attack_end(&mut h, enemy);
    h.advance(0.1); // 1st: 180（左拳）
    attack_end(&mut h, enemy);
    h.advance(0.1); // 2nd: 180 + 33 被动右拳

    let hp_before_third = h.health(enemy);
    attack_end(&mut h, enemy);
    h.advance(0.1); // 3rd: Q 已耗尽；左拳无被动 -> 0

    let dealt_third = hp_before_third - h.health(enemy);
    assert!(
        dealt_third.abs() < 1.0,
        "第 3 次普攻 Q 已耗尽，不应有 Q 附伤，实际 {}",
        dealt_third
    );
    h.finish();
}

/// Q 3 级 ratio 0.04，附伤 = 0.04×6000 = 240。
#[test]
fn sett_q_ratio_scales_with_level() {
    let mut h = build_headless("sett_q_level3");
    level_skill(&mut h, 0, 3);
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    h.cast_skill(0, Vec2::new(800.0, 0.0)).advance(0.1);
    attack_end(&mut h, enemy);
    h.advance(0.1);

    let dealt = hp_before - h.health(enemy);
    assert!(
        (dealt - 240.0).abs() < 1.0,
        "Q 3 级应附带 4% 目标最大生命=240，实际 {}",
        dealt
    );
    h.finish();
}
