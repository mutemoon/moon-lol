#![cfg(test)]

use bevy::math::{Vec2, Vec3};
use lol_core::movement::{CastBlock, MovementBlock};

use super::tests::build_headless;
use crate::riven::w::RIVEN_W_CAST_BLOCK_DURATION;

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

    // 系统只认标记：眩晕敌人身上应有 MovementBlock + CastBlock
    assert!(
        h.app.world().get::<MovementBlock>(enemy_near).is_some(),
        "范围内敌人应被 W 眩晕（MovementBlock）"
    );
    assert!(
        h.app.world().get::<CastBlock>(enemy_near).is_some(),
        "范围内敌人应被 W 眩晕（CastBlock）"
    );

    // 远距离敌人不被眩晕
    assert!(
        h.app.world().get::<MovementBlock>(enemy_far).is_none(),
        "范围外敌人不应被眩晕"
    );
    assert!(
        h.app.world().get::<CastBlock>(enemy_far).is_none(),
        "范围外敌人不应有 CastBlock"
    );

    // 等待眩晕过期（标记随 buff 死亡自动清除）
    h.advance(0.8);
    assert!(
        h.app.world().get::<MovementBlock>(enemy_near).is_none(),
        "0.75 秒后眩晕应过期（MovementBlock 移除）"
    );
    assert!(
        h.app.world().get::<CastBlock>(enemy_near).is_none(),
        "眩晕过期后 CastBlock 应移除"
    );

    h.finish();
}

/// Tests that W casting blocks other skills and movement for 8 frames (0.2667s)
#[test]
fn riven_w_casting_blocks_skill_and_movement() {
    let mut h = build_headless("riven_w_cast_block");

    // 施放 W
    h.cast_skill(1, Vec2::new(0.0, 0.0));
    h.advance(0.05); // 推进一点让系统处理事件

    // W 施法期间自阻塞：MovementBlock + CastBlock 由 BuffCastBlock 观察者桥接
    assert!(
        h.app.world().get::<MovementBlock>(h.champion).is_some(),
        "W 施法期间应有 MovementBlock"
    );
    assert!(
        h.app.world().get::<CastBlock>(h.champion).is_some(),
        "W 施法期间应有 CastBlock"
    );
    let q_entity = h.skill_entity(0);
    assert!(
        h.recast_window_stage(q_entity).is_none(),
        "Q 初始阶段应为 None"
    );

    // 尝试在阻塞期间施放 Q，应该被阻止
    h.cast_skill(0, Vec2::new(150.0, 0.0));
    h.advance(0.1);

    // 通过 stage 判断：被阻止的 Q 不应进入下一阶段（即不应创建 RecastWindow）
    assert!(
        h.recast_window_stage(q_entity).is_none(),
        "W 施法期间施放的 Q 不应进入下一阶段"
    );

    h.advance(RIVEN_W_CAST_BLOCK_DURATION + 0.05);

    // 阻塞结束后标记应移除
    assert!(
        h.app.world().get::<MovementBlock>(h.champion).is_none(),
        "W 施法阻塞结束后 MovementBlock 应移除"
    );
    assert!(
        h.app.world().get::<CastBlock>(h.champion).is_none(),
        "W 施法阻塞结束后 CastBlock 应移除"
    );

    // 此时施放 Q 应该成功
    h.cast_skill(0, Vec2::new(150.0, 0.0));
    h.advance(0.1);
    assert_eq!(
        h.recast_window_stage(q_entity),
        Some(2),
        "阻塞结束后施放 Q 应进入第 2 阶段"
    );

    h.finish();
}
