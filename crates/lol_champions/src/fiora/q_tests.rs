#![cfg(test)]

//! Fiora Q (Lunge) 单元测试。
//!
//! Q 的语义：向指针方向位移，位移停止后戳刺最近的单位；
//! 有敌方英雄时优先戳英雄，而不是像 Riven Q 那样对路径上的敌人造成碰撞伤害。

use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Entity, Transform};
use lol_core::base::bounding::Bounding;
use lol_core::life::Health;
use lol_core::team::Team;

use super::tests::build_headless;
use crate::test_utils::ChampionTestHarness;

const EPSILON: f32 = 1e-3;

/// 生成一个非英雄的敌方单位（小兵），用于测试“优先戳英雄”的目标筛选。
///
/// 测试 harness 的 `add_enemy` 只能生成英雄，这里手动生成一个不带 `Champion`
/// 组件的敌方实体来模拟小兵。
fn add_minion(h: &mut ChampionTestHarness, position: Vec3) -> Entity {
    h.app
        .world_mut()
        .spawn((
            Team::Chaos,
            Transform::from_translation(position),
            Health::new(6000.0),
            lol_core::damage::Armor(0.0),
        ))
        .id()
}

/// Q 应在位移停止后戳刺，而不是施法瞬间就造成伤害。
#[test]
fn fiora_q_strikes_after_dash_not_at_cast() {
    let mut h = build_headless("fiora_q_timing");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    // 位移 300 单位 @ 1000 速度 ≈ 0.3s；0.1s 时位移尚未结束，不应有戳刺伤害
    h.cast_skill(0, Vec2::new(300.0, 0.0)).advance(0.1);
    assert!(
        (h.health(enemy) - hp_before).abs() < EPSILON,
        "位移结束前不应造成伤害（戳刺发生在位移停止后），实际血量: {}",
        h.health(enemy)
    );

    // 位移结束后应戳刺最近单位
    h.advance(0.5);
    assert!(
        h.health(enemy) < hp_before,
        "位移停止后应戳刺最近敌人造成伤害"
    );

    h.finish();
}

/// 有敌方英雄时优先戳英雄，且不对位移路径上的敌人造成碰撞伤害（区别于 Riven Q）。
#[test]
fn fiora_q_prioritizes_champion_over_closer_minion() {
    let mut h = build_headless("fiora_q_priority");
    // 小兵在位移路径上，且更靠近位移终点 (300,0,0)
    let minion = add_minion(&mut h, Vec3::new(200.0, 0.0, 0.0));
    // 英雄在小兵侧面，距位移终点更远（约 180），但是英雄
    let champion = h.add_enemy(Vec3::new(200.0, 0.0, 150.0));

    let minion_hp_before = h.health(minion);
    let champ_hp_before = h.health(champion);

    h.cast_skill(0, Vec2::new(300.0, 0.0)).advance(0.1);
    // 位移路径上不应有碰撞伤害（区别于 Riven Q 的路径碰撞伤害）
    assert!(
        (h.health(minion) - minion_hp_before).abs() < EPSILON,
        "位移期间不应像 Riven 那样对路径上的敌人造成碰撞伤害，小兵血量: {}",
        h.health(minion)
    );

    h.advance(0.5);
    // 有英雄时优先戳英雄
    assert!(
        h.health(champion) < champ_hp_before,
        "有敌方英雄时应优先戳英雄"
    );
    assert!(
        (h.health(minion) - minion_hp_before).abs() < EPSILON,
        "优先戳英雄时不应误伤更近的小兵，小兵血量: {}",
        h.health(minion)
    );

    h.finish();
}

/// 没有敌方英雄时，戳刺最近的单位（小兵）。
#[test]
fn fiora_q_strikes_nearest_unit_when_no_champion() {
    let mut h = build_headless("fiora_q_fallback");
    let minion = add_minion(&mut h, Vec3::new(200.0, 0.0, 0.0));
    let hp_before = h.health(minion);

    h.cast_skill(0, Vec2::new(300.0, 0.0)).advance(0.1);
    assert!(
        (h.health(minion) - hp_before).abs() < EPSILON,
        "位移结束前不应造成伤害"
    );

    h.advance(0.5);
    assert!(
        h.health(minion) < hp_before,
        "没有敌方英雄时，应戳刺最近的单位（小兵）"
    );

    h.finish();
}

/// 戳刺命中判定应包含敌人的碰撞半径：以敌人边缘是否进入戳刺范围为准，
/// 而不是仅看敌人中心点。
#[test]
fn fiora_q_strike_includes_target_bounding_radius() {
    let mut h = build_headless("fiora_q_bounding");
    // 位移终点为 (300,0,0)。敌人中心距终点 220，刚好超出戳刺半径 200
    // （仅按中心点判定会 miss）；但敌人碰撞半径 35，边缘距离 220-35=185 在范围内，应命中。
    let enemy = h.add_enemy(Vec3::new(520.0, 0.0, 0.0));
    h.app.world_mut().entity_mut(enemy).insert(Bounding {
        radius: 35.0,
        height: 200.0,
    });

    let hp_before = h.health(enemy);

    h.cast_skill(0, Vec2::new(300.0, 0.0)).advance(0.1);
    assert!(
        (h.health(enemy) - hp_before).abs() < EPSILON,
        "位移结束前不应造成伤害"
    );

    h.advance(0.5);
    assert!(
        h.health(enemy) < hp_before,
        "戳刺命中判定应包含敌人碰撞半径（边缘进入范围即命中），敌人血量: {}",
        h.health(enemy)
    );

    h.finish();
}
