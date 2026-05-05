#![cfg(test)]

use bevy::math::{Vec2, Vec3};

use super::tests::{build_headless, riven_config};
use crate::riven::Riven;
use crate::test_utils::*;

const EPSILON: f32 = 1e-3;

#[test]
fn riven_e_spawns_shield_and_dash_absorbs_damage() {
    let mut h = build_headless("riven_e");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    h.cast_skill(2, Vec2::new(140.0, 0.0)).advance(0.4);

    assert!(h.position(h.champion).length() > 2.0, "E技能释放后应有位移");
    let initial_health = h.health(h.champion);
    let shield_val = h.shield_value().unwrap_or(0.0);

    // 护盾值从 spell data 读取
    let expected_shield = h
        .get_skill_value(
            2,
            "total_shield",
            1,
            |stat| {
                if stat == 2 { 64.0 } else { 0.0 }
            },
        )
        .expect("shield value should exist from spell data");

    assert!(
        (shield_val - expected_shield).abs() < EPSILON,
        "E 护盾值 ({} 应等于技能数据 {})",
        shield_val,
        expected_shield
    );

    h.apply_damage(enemy, 60.0);

    assert!(
        (h.health(h.champion) - initial_health).abs() < EPSILON,
        "60点伤害应被护盾完全吸收"
    );
    let remaining_shield = h.shield_value().unwrap_or(0.0);
    assert!(
        remaining_shield > 0.0 && remaining_shield < shield_val,
        "护盾消耗后应剩余部分值"
    );

    h.apply_damage(enemy, 9999.0);

    assert!(
        h.health(h.champion) < initial_health,
        "护盾耗尽后，生命值应下降"
    );
    h.finish();
}
