#![cfg(test)]

use bevy::math::{Vec2, Vec3};
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::Buffs;
use lol_core::base::level::Level;
use lol_core::damage::Damage;

use crate::riven::passive::{passive_ratio_for_level, BuffRivenPassive};
use crate::riven::tests::build_headless;
use crate::test_utils::*;

const EPSILON: f32 = 1e-3;

/// 读取锐雯当前被动层数（无被动 buff 返回 None）
fn passive_charges(h: &ChampionTestHarness) -> Option<u8> {
    let world = h.app.world();
    let buffs = world.get::<Buffs>(h.champion)?;
    for buff_entity in buffs.iter() {
        if let Some(passive) = world.get::<BuffRivenPassive>(*buff_entity) {
            return Some(passive.charges);
        }
    }
    None
}

/// 施放 E（纯位移+护盾，不造成伤害）应获得1层被动
#[test]
fn riven_passive_grants_charge_on_skill_cast() {
    let mut h = build_headless("riven_passive_grant");
    assert_eq!(passive_charges(&h), None, "初始无被动层数");

    h.cast_skill(2, Vec2::new(140.0, 0.0)).advance(0.4);

    assert_eq!(passive_charges(&h), Some(1), "施放技能后应获得1层被动");
    h.finish();
}

/// 多次施法叠加被动，但上限为3层
#[test]
fn riven_passive_caps_at_three() {
    let mut h = build_headless("riven_passive_cap");

    // Q 三段 + E = 4 次施法，被动上限 3 层
    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.4);
    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.4);
    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.4);
    h.cast_skill(2, Vec2::new(100.0, 0.0)).advance(0.4);

    assert_eq!(passive_charges(&h), Some(3), "被动最多叠加3层");
    h.finish();
}

/// 普攻命中消耗1层被动并造成额外伤害（额外伤害 = AD * 等级倍率）
#[test]
fn riven_passive_consumes_charge_for_bonus() {
    let mut h = build_headless("riven_passive_consume");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    h.cast_skill(2, Vec2::new(140.0, 0.0)).advance(0.4);
    assert_eq!(passive_charges(&h), Some(1));

    let hp_before = h.health(enemy);
    let ad = h.app.world().get::<Damage>(h.champion).unwrap().0;
    let level = h.app.world().get::<Level>(h.champion).unwrap().value;
    let expected_bonus = ad * passive_ratio_for_level(level);

    // 直接触发 EventAttackEnd 只触发 on-hit 观察者（被动额外伤害），
    // 不含基础普攻伤害，因此掉血量即被动额外伤害
    h.app.world_mut().trigger(EventAttackEnd {
        entity: h.champion,
        target: enemy,
    });
    h.advance(0.1);

    let hp_after = h.health(enemy);
    let actual = hp_before - hp_after;
    assert!(
        (actual - expected_bonus).abs() < EPSILON,
        "被动消耗应造成额外伤害 {:.2}（AD={} level={} ratio={:.4}），实际掉血 {:.2}",
        expected_bonus,
        ad,
        level,
        passive_ratio_for_level(level),
        actual
    );
    assert_eq!(passive_charges(&h), Some(0), "命中后应消耗1层被动");
    h.finish();
}

/// 无被动层数时，普攻命中不造成额外伤害
#[test]
fn riven_passive_no_bonus_without_charges() {
    let mut h = build_headless("riven_passive_no_charge");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    let hp_before = h.health(enemy);
    h.app.world_mut().trigger(EventAttackEnd {
        entity: h.champion,
        target: enemy,
    });
    h.advance(0.1);

    assert!(
        (h.health(enemy) - hp_before).abs() < EPSILON,
        "无被动层数时不应造成额外伤害"
    );
    h.finish();
}

/// 被动持续6秒，超时后层数清空、buff 消失
#[test]
fn riven_passive_expires_after_six_seconds() {
    let mut h = build_headless("riven_passive_expire");

    h.cast_skill(2, Vec2::new(140.0, 0.0)).advance(0.4);
    assert_eq!(passive_charges(&h), Some(1));

    h.advance(6.2);

    assert_eq!(passive_charges(&h), None, "被动6秒后应消失");
    h.finish();
}

/// 连续施法刷新被动持续时间：先充能，临近过期时再施法应重置计时
#[test]
fn riven_passive_refreshes_on_subsequent_cast() {
    let mut h = build_headless("riven_passive_refresh");

    h.cast_skill(2, Vec2::new(140.0, 0.0)).advance(5.5);
    assert_eq!(passive_charges(&h), Some(1), "5.5秒后被动仍存在");

    // 再次施法刷新计时器，再过 1 秒（距首次施法 6.5 秒）被动应仍在
    h.cast_skill(1, Vec2::new(100.0, 0.0)).advance(1.0);
    assert_eq!(
        passive_charges(&h),
        Some(2),
        "二次施法应叠加为2层并刷新计时，1秒后未过期"
    );

    // 再过 5.5 秒（距二次施法 6.5 秒）应过期
    h.advance(5.5);
    assert_eq!(passive_charges(&h), None, "刷新后6秒应过期");
    h.finish();
}
