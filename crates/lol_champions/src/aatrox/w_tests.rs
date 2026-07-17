#![cfg(test)]

use bevy::math::{Vec2, Vec3};

use super::tests::*;

/// W 命中：首段伤害 20 + 0.4*60 = 44，并附加减速与标记。
#[test]
fn w_damages_and_slows() {
    let mut h = build_headless("aatrox_w_first");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    h.advance(0.1);

    let hp_before = h.health(enemy);
    h.cast_skill(1, Vec2::new(200.0, 0.0)).advance(0.1);
    let dealt = hp_before - h.health(enemy);

    assert!((dealt - 44.0).abs() < 1.5, "W 首段应为 44，实际 {dealt}");
    assert!(is_slowed(&h, enemy), "W 应附加减速");
    assert!(has_mark(&h, enemy), "W 应附加引爆标记");
}

/// W 标记 1.5s 后引爆，造成等额二次伤害（总 88）。
#[test]
fn w_second_hit_after_delay() {
    let mut h = build_headless("aatrox_w_second");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    h.advance(0.1);

    let hp_before = h.health(enemy);
    h.cast_skill(1, Vec2::new(200.0, 0.0)).advance(0.1);
    // 首段 44；标记 1.5s 后引爆再 44。
    h.advance(1.6);
    let total = hp_before - h.health(enemy);

    assert!(
        (total - 88.0).abs() < 3.0,
        "W 两段总伤害应为 88，实际 {total}"
    );
}

/// W 标记引爆时拉回中心：目标应被移动到 Aatrox 脚下附近。
#[test]
fn w_pulls_to_center_on_explosion() {
    let mut h = build_headless("aatrox_w_pull");
    // 敌人在 (500, 0, 0)，Aatrox 在原点
    let enemy = h.add_enemy(Vec3::new(500.0, 0.0, 0.0));
    h.advance(0.1);

    let pos_before = h.position(enemy);
    h.cast_skill(1, Vec2::new(500.0, 0.0)).advance(0.1);
    // 等待标记引爆
    h.advance(2.0);
    let pos_after = h.position(enemy);

    // 敌人应被拉回到 Aatrox 附近（原点 ±150）
    assert!(
        pos_after.x.abs() < 150.0,
        "W 标记引爆应将敌人拉回中心，{:.1} → {:.1}",
        pos_before.x,
        pos_after.x
    );
}
#[test]
fn w_knockup_after_delay() {
    let mut h = build_headless("aatrox_w_knockup");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    h.advance(0.1);

    h.cast_skill(1, Vec2::new(200.0, 0.0)).advance(0.1);
    assert!(!is_knockup(&h, enemy), "W 首段不应击飞");

    h.advance(1.6);
    assert!(is_knockup(&h, enemy), "W 标记引爆应附加击飞");
}
