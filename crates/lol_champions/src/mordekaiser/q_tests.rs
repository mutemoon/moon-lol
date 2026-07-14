#![cfg(test)]

use bevy::math::{Vec2, Vec3};
use lol_core::action::delayed_damage::DelayedDamageInstance;

use super::tests::build_headless;
use crate::test_utils::*;

/// 矩形内敌人应在延迟结束后受到魔法伤害；延迟结束前不受伤。
#[test]
fn mordekaiser_q_deals_damage_after_delay() {
    let mut h = build_headless("morde_q_damage_after_delay");
    // 敌人在前方 +X 600 处（矩形 [400,1025]×[-80,80] 内）
    let enemy = h.add_enemy(Vec3::new(600.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    // 延迟 0.3s：advance 0.1s 时不应有伤害
    h.cast_skill(0, Vec2::new(800.0, 0.0)).advance(0.1);
    assert!(
        (h.health(enemy) - hp_before).abs() < 0.01,
        "延迟结束前不应造成伤害，实际血量变化 {}",
        hp_before - h.health(enemy)
    );

    // 再 advance 到 0.5s（超过延迟），应有伤害
    h.advance(0.4);
    assert!(
        h.health(enemy) < hp_before,
        "延迟结束后矩形内敌人应受到伤害"
    );
    h.finish();
}

/// 矩形外的敌人（死区 / 横向超出半宽）不应受到伤害。
#[test]
fn mordekaiser_q_ignores_outside_rectangle() {
    let mut h = build_headless("morde_q_outside");
    // 死区（< 400）：敌人在 300 处，矩形从 400 开始
    let enemy_deadzone = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    // 横向偏离 +X 方向 200（矩形半宽 80，超出）
    let enemy_lateral = h.add_enemy(Vec3::new(600.0, 0.0, 200.0));
    let hp_deadzone = h.health(enemy_deadzone);
    let hp_lateral = h.health(enemy_lateral);

    h.cast_skill(0, Vec2::new(800.0, 0.0)).advance(0.5);

    assert!(
        (h.health(enemy_deadzone) - hp_deadzone).abs() < 0.01,
        "死区内敌人不应受伤"
    );
    assert!(
        (h.health(enemy_lateral) - hp_lateral).abs() < 0.01,
        "横向超出矩形宽度的敌人不应受伤"
    );
    h.finish();
}

/// 孤立目标（矩形内仅 1 个）伤害应高于多目标情况（IsolationScalar 乘算）。
#[test]
fn mordekaiser_q_isolation_bonus() {
    let mut h = build_headless("morde_q_isolation");

    // 第一发：两个敌人都在矩形内 -> 多目标，无孤立加成
    let e1 = h.add_enemy(Vec3::new(600.0, 0.0, 0.0));
    let e2 = h.add_enemy(Vec3::new(700.0, 0.0, 0.0));
    let hp1_before = h.health(e1);
    h.cast_skill(0, Vec2::new(800.0, 0.0)).advance(0.5);
    let multi_damage = hp1_before - h.health(e1);

    // 等 Q 冷却（8s）后移除一个敌人再施法 -> 孤立，有加成
    h.advance(8.5);
    h.app.world_mut().entity_mut(e2).despawn();
    let hp1_before2 = h.health(e1);
    h.cast_skill(0, Vec2::new(800.0, 0.0)).advance(0.5);
    let solo_damage = hp1_before2 - h.health(e1);

    assert!(
        solo_damage > multi_damage * 1.15,
        "孤立目标伤害应明显高于多目标：solo={}, multi={}",
        solo_damage,
        multi_damage
    );
    assert!(
        multi_damage > 0.0,
        "多目标也应受到伤害（无加成），multi={}",
        multi_damage
    );
    h.finish();
}

/// 施法后应生成 DelayedDamageInstance 指示器实体，并在延迟+褪去后销毁。
#[test]
fn mordekaiser_q_indicator_lifecycle() {
    let mut h = build_headless("morde_q_indicator");
    h.cast_skill(0, Vec2::new(500.0, 0.0)).advance(0.05);

    let spawned = {
        let mut q = h.app.world_mut().query::<&DelayedDamageInstance>();
        q.iter(h.app.world()).count()
    };
    assert!(spawned >= 1, "施法后应生成延迟伤害指示器实体");

    // 推进超过 延迟(0.3) + 褪去(0.3)，指示器应已销毁
    h.advance(0.8);
    let remaining = {
        let mut q = h.app.world_mut().query::<&DelayedDamageInstance>();
        q.iter(h.app.world()).count()
    };
    assert_eq!(remaining, 0, "延迟+褪去结束后指示器应销毁");
    h.finish();
}
