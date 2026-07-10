#![cfg(test)]

//! Fiora E（利刃之舞 / Bladework）单元测试。
//!
//! E：获得额外攻速（3s），下两次普攻增强——第一击减速、第二击必暴（150%伤害）。
//! 两击消耗后或 3s 后 buff 结束并移除攻速加成。

use bevy::math::Vec3;
use bevy::prelude::Entity;
use lol_core::attack::{BuffAttack, EventAttackEnd};
use lol_core::base::buff::Buffs;
use lol_core::buffs::cc_debuffs::DebuffSlow;

use super::tests::build_headless;
use crate::fiora::e::BuffFioraE;
use crate::test_utils::ChampionTestHarness;

/// E 应按技能等级赋予额外攻速（ron ASPercent，1 级 = 0.4）。
#[test]
fn fiora_e_grants_attack_speed() {
    let mut h = build_headless("fiora_e_as");
    h.cast_skill(2, bevy::math::Vec2::ZERO).advance(0.1);

    let as_bonus = h
        .app
        .world()
        .get::<BuffAttack>(h.champion)
        .expect("E 后菲奥娜应有 BuffAttack")
        .bonus_attack_speed;
    assert!(
        (as_bonus - 0.4).abs() < 1e-3,
        "E 应按等级赋予 0.4 攻速，实际 {:.3}",
        as_bonus
    );

    h.finish();
}

/// E 的第一击应减速目标（40%，1s）。
#[test]
fn fiora_e_first_hit_slows() {
    let mut h = build_headless("fiora_e_slow");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    h.cast_skill(2, bevy::math::Vec2::ZERO).advance(0.1);
    // 直接触发一次攻击结束事件，模拟第一击命中
    h.app.world_mut().trigger(EventAttackEnd {
        entity: h.champion,
        target: enemy,
    });
    h.advance(0.1);

    assert!(has_debuff_slow(&h, enemy), "E 第一击应减速目标");

    h.finish();
}

/// E 的第二击应造成额外暴击伤害（AttackTwoPercentTAD - 1)×AD，1 级约 0.5×66=33）。
#[test]
fn fiora_e_second_hit_crits() {
    let mut h = build_headless("fiora_e_crit");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    h.cast_skill(2, bevy::math::Vec2::ZERO).advance(0.1);

    let hp_before = h.health(enemy);
    // 触发两次攻击结束：第一击（减速，无额外伤害）+ 第二击（暴击额外伤害）
    h.app.world_mut().trigger(EventAttackEnd {
        entity: h.champion,
        target: enemy,
    });
    h.advance(0.1);
    let hp_after_first = h.health(enemy);

    h.app.world_mut().trigger(EventAttackEnd {
        entity: h.champion,
        target: enemy,
    });
    h.advance(0.1);
    let hp_after_second = h.health(enemy);

    // 第一击不应有额外伤害（仅减速），第二击应有暴击额外伤害
    let first_extra = hp_before - hp_after_first;
    let second_extra = hp_after_first - hp_after_second;
    assert!(
        first_extra < 1.0,
        "第一击不应有额外伤害，实际 {:.1}",
        first_extra
    );
    assert!(
        second_extra > 20.0,
        "第二击应造成额外暴击伤害，实际 {:.1}",
        second_extra
    );

    h.finish();
}

/// E 应在持续时间（3s）后过期，移除 buff 与攻速加成。
#[test]
fn fiora_e_expires_after_duration() {
    let mut h = build_headless("fiora_e_expire");
    h.cast_skill(2, bevy::math::Vec2::ZERO).advance(0.1);
    assert!(has_e_buff(&h), "E 施法后应立刻挂上 BuffFioraE");
    assert!(
        h.app.world().get::<BuffAttack>(h.champion).is_some(),
        "E 施法后应有攻速加成"
    );

    h.advance(3.5); // 超过 3s 持续时间

    assert!(!has_e_buff(&h), "E 应在 3s 后过期，移除 BuffFioraE");
    assert!(
        h.app.world().get::<BuffAttack>(h.champion).is_none(),
        "E 过期后应移除攻速加成"
    );

    h.finish();
}

fn has_e_buff(h: &ChampionTestHarness) -> bool {
    let world = h.app.world();
    let Some(buffs) = world.get::<Buffs>(h.champion) else {
        return false;
    };
    buffs
        .iter()
        .any(|be| world.get::<BuffFioraE>(*be).is_some())
}

/// 目标是否被挂上 DebuffSlow（buff 是独立实体，需遍历目标的 Buffs）。
fn has_debuff_slow(h: &ChampionTestHarness, target: Entity) -> bool {
    let world = h.app.world();
    let Some(buffs) = world.get::<Buffs>(target) else {
        return false;
    };
    buffs
        .iter()
        .any(|be| world.get::<DebuffSlow>(*be).is_some())
}
