#![cfg(test)]

use bevy::math::{Vec2, Vec3};

use super::tests::{build_headless, build_render};

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

#[test]
fn riven_q_field_damages_enemy_once() {
    let mut h = build_headless("riven_q_field_damage");
    let enemy = h.add_enemy(Vec3::new(150.0, 0.0, 0.0));

    let hp_before = h.health(enemy);
    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.1);

    // 第一次检查：敌人应已受到伤害
    let hp_first = h.health(enemy);
    assert!(
        hp_first < hp_before,
        "Q1 的 RivenQField 应对附近敌人造成伤害 ({} < {})",
        hp_first,
        hp_before
    );

    // 持续前进：同一字段不应重复伤害同一敌人
    h.advance(0.3);
    assert_eq!(
        h.health(enemy),
        hp_first,
        "RivenQField 不应重复伤害同一敌人"
    );

    // 等待字段过期并验证字段已销毁
    h.advance(1.0);
    let field_exists = {
        let world = h.app.world_mut();
        let mut q = world.query::<&lol_core::missile::AttachedField>();
        q.iter(world).next().is_some()
    };
    assert!(!field_exists, "AttachedField 应在计时结束后销毁");

    h.finish();
}

#[test]
fn riven_q_field_spawns_per_stage() {
    let mut h = build_headless("riven_q_field_stages");
    let enemy = h.add_enemy(Vec3::new(150.0, 0.0, 0.0));
    let q_entity = h.skill_entity(0);

    // Q1 释放
    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.1);
    let hp_after_q1 = h.health(enemy);
    assert!(hp_after_q1 < 6000.0, "Q1 field should deal damage");

    // 字段应在 0.5 秒后销毁
    h.advance(1.0);
    let field_removed = {
        let world = h.app.world_mut();
        let mut q = world.query::<&lol_core::missile::AttachedField>();
        q.iter(world).next().is_none()
    };
    assert!(field_removed, "Q1 field should be despawned before Q2");

    // Q2 释放
    assert_eq!(h.recast_window_stage(q_entity), Some(2), "Q1 后应为第2阶段");
    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.1);
    let hp_after_q2 = h.health(enemy);
    assert!(hp_after_q2 < hp_after_q1, "Q2 field should deal damage");

    // 字段销毁
    h.advance(1.0);

    // Q3 释放
    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.1);
    let hp_after_q3 = h.health(enemy);
    assert!(hp_after_q3 < hp_after_q2, "Q3 field should deal damage");

    h.advance(1.0);
    h.finish();
}

/// Q1 释放后，虽然主冷却（13s）已开始计时，但在 4s 重施窗口内
/// 技能应显示为"就绪（可释放 Q2）"而非直接显示冷却倒计时。
/// 重施窗口过期后，主冷却仍在进行时才应显示冷却。
#[test]
fn riven_q_shows_ready_during_recast_window() {
    let mut h = build_headless("riven_q_display_ready");
    let q_entity = h.skill_entity(0);

    // 初始：无冷却、无重施窗口 -> 就绪
    assert!(h.is_skill_ready(q_entity), "初始 Q 应显示为就绪");

    // Q1 释放：AfterCast 模式下立即进入 13s 主冷却，同时开启 4s 的 Q2 重施窗口
    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.4);

    assert!(h.has_recast_window(q_entity), "Q1 后应存在重施窗口");
    // 关键：重施窗口内虽然冷却在计时，但应显示"可释放 Q2"而非 CD
    assert!(
        h.is_skill_ready(q_entity),
        "重施窗口内 Q 应显示为就绪（可释放 Q2），而非直接显示 CD"
    );

    // 重施窗口 4s 过期，但主冷却（13s）仍在计时 -> 应显示 CD
    h.advance(4.0);
    assert!(!h.has_recast_window(q_entity), "4s 后重施窗口应消失");
    assert!(
        !h.is_skill_ready(q_entity),
        "重施窗口过期后冷却仍未结束，应显示 CD"
    );

    h.finish();
}
