#![cfg(test)]

use bevy::math::{Vec2, Vec3};
use lol_core::movement::MovementBlock;

use super::tests::{build_headless, riven_config};
use crate::riven::{BuffStun, Riven};
use crate::test_utils::*;

const EPSILON: f32 = 1e-3;

#[test]
fn riven_w_hits_only_enemies_in_range() {
    let mut h = build_headless("riven_w");
    let enemy_near = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    let enemy_far = h.add_enemy(Vec3::new(420.0, 0.0, 0.0));
    let ally_near = h.add_ally(Vec3::new(60.0, 0.0, 0.0));
    let expected_damage = h
        .get_skill_value(
            1,
            "total_damage",
            1,
            |stat| {
                if stat == 2 { 64.0 } else { 0.0 }
            },
        )
        .expect("riven w damage should exist");
    let initial_near = h.health(enemy_near);
    let initial_far = h.health(enemy_far);
    let initial_ally = h.health(ally_near);

    h.cast_skill(1, Vec2::new(140.0, 0.0));

    h.advance(0.2);

    assert!(
        (initial_near - h.health(enemy_near) - expected_damage).abs() < EPSILON,
        "近距离敌人应受到W技能全额伤害"
    );
    assert!(
        (h.health(enemy_far) - initial_far).abs() < EPSILON,
        "远处敌人（距420）应在W范围外不受伤害"
    );
    assert!(
        (h.health(ally_near) - initial_ally).abs() < EPSILON,
        "友军不应受W技能影响"
    );
    h.finish();
}

#[test]
fn riven_w_stuns_enemies_in_range() {
    let mut h = build_headless("riven_w_stun");
    let enemy_near = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    let enemy_far = h.add_enemy(Vec3::new(420.0, 0.0, 0.0));

    h.cast_skill(1, Vec2::new(140.0, 0.0)).advance(0.1);

    // 近距离敌人被眩晕
    assert!(
        h.app.world().get::<BuffStun>(enemy_near).is_some(),
        "范围内敌人应被 W 眩晕"
    );
    assert!(
        h.app.world().get::<MovementBlock>(enemy_near).is_some(),
        "眩晕敌人应有 MovementBlock"
    );

    // 远距离敌人不被眩晕
    assert!(
        h.app.world().get::<BuffStun>(enemy_far).is_none(),
        "范围外敌人不应被眩晕"
    );

    // 等待眩晕过期
    h.advance(0.8);
    assert!(
        h.app.world().get::<BuffStun>(enemy_near).is_none(),
        "0.75 秒后眩晕应过期"
    );
    assert!(
        h.app.world().get::<MovementBlock>(enemy_near).is_none(),
        "眩晕过期后 MovementBlock 应移除"
    );

    h.finish();
}
