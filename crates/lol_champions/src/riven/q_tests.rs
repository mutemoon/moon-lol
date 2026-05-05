#![cfg(test)]

use bevy::math::{Vec2, Vec3};

use super::tests::{build_headless, build_render, riven_config};
use crate::riven::Riven;
use crate::test_utils::*;

#[test]
fn riven_q_cycles_through_three_real_stages() {
    let mut h = build_render("riven_q");

    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(3.5);

    let q_entity = h.skill_entity(0);

    assert_eq!(
        h.recast_window_stage(q_entity),
        Some(2),
        "第一次Q释放后应为第2阶段"
    );
    assert!(h.can_cast(0), "Q技能应该可以释放第二段");

    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(3.5);

    assert_eq!(
        h.recast_window_stage(q_entity),
        Some(3),
        "第二次Q释放后应为第3阶段"
    );
    assert!(h.can_cast(0), "Q技能应该可以释放第3段");

    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(1.0);

    assert!(
        !h.has_recast_window(q_entity),
        "Q技能三段全用完，RecastWindow应被移除"
    );
    assert!(!h.can_cast(0), "Q技能三段不能再释放");

    h.advance(6.0);
    h.finish();

    assert!(h.can_cast(0), "等待足够时间后，冷却应已结束");
    assert!(
        h.position(h.champion).length() > 5.0,
        "三段Q位移后离原点应超过5单位"
    );
}

#[test]
fn riven_q_recast_window_expires_after_4_seconds() {
    let mut h = build_headless("riven_q_window");

    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.4);

    let q_entity = h.skill_entity(0);

    assert_eq!(
        h.recast_window_stage(q_entity),
        Some(2),
        "第一次Q释放后应为第2阶段"
    );

    h.advance(3.5);
    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.1);

    assert_eq!(
        h.recast_window_stage(q_entity),
        Some(3),
        "3.9秒时释放Q2，应进入第3阶段"
    );

    h.advance(0.15);
    assert!(
        h.has_recast_window(q_entity),
        "Q2创建了新窗口，新的4秒计时器未到期"
    );

    h.advance(3.9);
    assert!(!h.has_recast_window(q_entity), "Q2的4秒窗口到期消失");
    h.finish();
}

#[test]
fn riven_q3_knocks_back_enemies() {
    let mut h = build_headless("riven_q3_knockback");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let enemy_pos_before = h.position(enemy);

    // 三段 Q 朝向敌人
    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.4);
    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.4);
    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.5);

    let enemy_pos_after = h.position(enemy);
    let distance_moved = (enemy_pos_after - enemy_pos_before).length();

    assert!(
        distance_moved > 10.0,
        "Q3 应击退范围内敌人（移动距离: {}）",
        distance_moved
    );

    // 验证 Riven 的 RivenQ3Pending 已清除
    assert!(
        h.app
            .world()
            .get::<crate::riven::buffs::RivenQ3Pending>(h.champion)
            .is_none(),
        "Q3 位移结束后 RivenQ3Pending 应被移除"
    );

    h.finish();
}
