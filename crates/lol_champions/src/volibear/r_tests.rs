#![cfg(test)]

//! Volibear R（风暴之怒）集成测试。
//!
//! R：向指针方向突进（最大 550），落地后造成 sweet_spot_damage 物理伤害并减速，
//! 同时获得额外生命值。
//! sweet_spot_damage @1级 = 100 + 2.5*AD = 100 + 162.5 = 262.5。
//! 额外生命 @2级(idx1) = 175。

use bevy::math::{Vec2, Vec3};

use super::tests::{build_headless, is_slowed, level_skill, max_health};

/// R 落地应对范围内敌人造成 262.5 物理伤害。
#[test]
fn volibear_r_landing_damage() {
    let mut h = build_headless("volibear_r_damage");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.6);

    let dealt = hp_before - h.health(enemy);
    assert!(
        (dealt - 262.5).abs() < 2.0,
        "R 落地应造成 262.5 物理伤害，实际 {dealt}"
    );
    h.finish();
}

/// R 落地应减速范围内敌人 50%，持续 1s。
#[test]
fn volibear_r_landing_slow() {
    let mut h = build_headless("volibear_r_slow");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.6);

    assert!(is_slowed(&h, enemy), "R 落地应减速敌人");
    h.finish();
}

/// R 应增加沃利贝尔最大生命值（@2级 +175）。
#[test]
fn volibear_r_grants_bonus_health() {
    let mut h = build_headless("volibear_r_health");
    let champion = h.champion;
    level_skill(&mut h, 3, 2);
    let hp_before = max_health(&h, champion);

    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.6);

    let hp_after = max_health(&h, champion);
    assert!(
        (hp_after - hp_before - 175.0).abs() < 1.0,
        "R @2级应增加 175 最大生命，前 {hp_before} 后 {hp_after}"
    );
    h.finish();
}
