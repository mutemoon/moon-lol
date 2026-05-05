#![cfg(test)]

use bevy::math::{Vec2, Vec3};
use lol_core::attack::Attack;
use lol_core::damage::Damage;
use lol_core::life::Health;

use super::tests::{build_headless, riven_config};
use crate::riven::Riven;
use crate::test_utils::*;

const EPSILON: f32 = 1e-3;

#[test]
fn riven_r_buff_increases_stats_and_allows_recast() {
    let mut h = build_headless("riven_r_buff");
    let base_damage = h.app.world().get::<Damage>(h.champion).unwrap().0;
    let base_range = h.app.world().get::<Attack>(h.champion).unwrap().range;

    h.cast_skill(3, Vec2::new(140.0, 0.0)).advance(0.2);

    let buff_damage = h.app.world().get::<Damage>(h.champion).unwrap().0;
    let buff_range = h.app.world().get::<Attack>(h.champion).unwrap().range;

    assert!(
        (buff_damage - base_damage * 1.25).abs() < EPSILON,
        "R 开启后 AD 应增加 25%（从 {} 到 {})",
        base_damage,
        buff_damage
    );
    assert!(
        (buff_range - base_range - 75.0).abs() < EPSILON,
        "R 开启后攻击距离应增加 75"
    );

    // 可重施 Wind Slash
    assert!(
        h.has_recast_window(h.skill_entity(3)),
        "R 开启后应有连招窗口"
    );
    assert!(h.can_cast(3), "R 开启后应可重施 Wind Slash");

    h.finish();
}

#[test]
fn riven_r_wind_slash_deals_damage_in_cone() {
    let mut h = build_headless("riven_r_wind_slash");
    // 默认 forward 方向是 -Z，敌人放在 -Z 方向
    let enemy_front = h.add_enemy(Vec3::new(0.0, 0.0, -300.0));
    let enemy_behind = h.add_enemy(Vec3::new(0.0, 0.0, 300.0));
    let initial_hp = h.health(enemy_front);

    // R 初次
    h.cast_skill(3, Vec2::ZERO).advance(0.2);
    // Wind Slash（R 重施）
    h.cast_skill(3, Vec2::new(0.0, -300.0)).advance(0.2);

    assert!(
        h.health(enemy_front) < initial_hp,
        "Wind Slash 应对前方 -Z 方向敌人造成伤害"
    );
    assert!(
        (h.health(enemy_behind) - 6000.0).abs() < EPSILON,
        "Wind Slash 不应伤害身后 +Z 方向敌人"
    );

    // R 进入冷却
    assert!(!h.can_cast(3), "Wind Slash 后 R 应进入冷却");

    h.finish();
}

#[test]
fn riven_r_wind_slash_deals_more_damage_to_low_hp_targets() {
    let mut h = build_headless("riven_r_wind_slash_missing_hp");
    // 默认 forward 方向是 -Z
    let enemy_low = h.add_enemy(Vec3::new(0.0, 0.0, -300.0));
    let enemy_full = h.add_enemy(Vec3::new(0.0, 0.0, -500.0));

    // 设置低血量敌人（10% HP）
    h.app.world_mut().entity_mut(enemy_low).insert(Health {
        value: 600.0,
        max: 6000.0,
    });

    let initial_low = h.health(enemy_low);

    h.cast_skill(3, Vec2::ZERO).advance(0.2);
    h.cast_skill(3, Vec2::new(0.0, -300.0)).advance(0.2);

    let damage_low = initial_low - h.health(enemy_low);
    let damage_full = 6000.0 - h.health(enemy_full);

    assert!(
        damage_low > 0.0,
        "低血量敌人应受到 Wind Slash 伤害 ({})",
        damage_low
    );
    assert!(
        damage_full > 0.0,
        "满血敌人也应受到 Wind Slash 伤害 ({})",
        damage_full
    );
    assert!(
        (damage_low - damage_full).abs() < EPSILON,
        "导弹系统使用平均 HP 计算伤害，两个敌人应受到相近伤害 (低: {}, 满: {})",
        damage_low,
        damage_full
    );

    h.finish();
}

#[test]
fn riven_r_buff_expires_after_15_seconds() {
    let mut h = build_headless("riven_r_buff_expire");
    let base_damage = h.app.world().get::<Damage>(h.champion).unwrap().0;

    h.cast_skill(3, Vec2::ZERO).advance(0.2);

    // 15 秒后 buff 过期
    h.advance(16.0);

    let final_damage = h.app.world().get::<Damage>(h.champion).unwrap().0;

    assert!(
        (final_damage - base_damage).abs() < EPSILON,
        "R buff 过期后 AD 应恢复基础值 ({} -> {})",
        base_damage,
        final_damage
    );
    h.finish();
}

#[test]
fn riven_r_starts_cooldown_without_moving_or_damaging() {
    let mut h = build_headless("riven_r_no_move");
    let _enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    h.cast_skill(3, Vec2::new(140.0, 0.0)).advance(0.2);

    // 初始 R 不应移动
    assert!(
        h.position(h.champion).distance(Vec3::ZERO) < EPSILON,
        "初始 R 释放后位置不应移动"
    );

    // 应可重施 Wind Slash
    assert!(h.can_cast(3), "初始 R 后可重施 Wind Slash（不冷却）");

    h.finish();
}
