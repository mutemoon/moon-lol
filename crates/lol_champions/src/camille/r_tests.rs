#![cfg(test)]

//! Camille R（海克斯最后通牒 / Hextech Ultimatum）集成测试（TDD）。
//!
//! 简化实现：R 标记最近的敌方英雄，普攻命中被标记目标时造成额外魔法伤害
//! （`RPercentCurrentHPDamage`% 当前生命值，1 级 = 2%），持续 `RDuration`（1.75s）。
//! 区域禁锢 / 击退按位移框架 Phase 4.2 暂缓。

use bevy::math::{Vec2, Vec3};
use bevy::prelude::Entity;
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::Buffs;

use super::tests::build_headless;
use crate::camille::r::BuffCamilleRMark;
use crate::test_utils::ChampionTestHarness;

/// 读取目标是否带有 R 标记（遍历其 Buffs）。
fn has_r_mark(h: &ChampionTestHarness, target: Entity) -> bool {
    let Some(buffs) = h.app.world().get::<Buffs>(target) else {
        return false;
    };
    buffs
        .iter()
        .any(|b| h.app.world().get::<BuffCamilleRMark>(*b).is_some())
}

/// 手动触发一次普攻命中（仅触发 on-hit，不造成基础 AA 伤害）。
fn attack_end(h: &mut ChampionTestHarness, target: Entity) {
    h.app.world_mut().trigger(EventAttackEnd {
        entity: h.champion,
        target,
    });
}

/// R 应标记距施法点最近的敌方英雄。
#[test]
fn camille_r_marks_target() {
    let mut h = build_headless("camille_r_mark");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let mana_before = h.mana();

    h.cast_skill(3, Vec2::new(200.0, 0.0)).advance(0.2);
    assert!(has_r_mark(&h, enemy), "R 应标记施法点附近的敌方英雄");
    assert!(
        !h.can_cast(3),
        "R 施放后应进入冷却"
    );
    assert!(h.mana() < mana_before, "R 施放应消耗法力");
    h.finish();
}

/// 普攻命中被标记目标应造成额外魔法伤害（2% 当前生命值）。
#[test]
fn camille_r_bonus_damage_on_marked_target() {
    let mut h = build_headless("camille_r_bonus");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));

    // 施放 R：标记 + 冲刺物理伤害
    h.cast_skill(3, Vec2::new(200.0, 0.0)).advance(0.5);
    assert!(has_r_mark(&h, enemy), "R 后目标应被标记");

    let hp_at_hit = h.health(enemy);
    attack_end(&mut h, enemy); // 触发 R 标记额外魔法伤害
    h.advance(0.1);

    let dealt = hp_at_hit - h.health(enemy);
    let expected = hp_at_hit * 0.02;
    assert!(
        (dealt - expected).abs() < 2.0,
        "命中被标记目标应造成 ≈{:.1} 额外魔法伤害（2% 当前生命值），实际 {:.1}",
        expected,
        dealt
    );
    h.finish();
}

/// 标记应在 RDuration（1.75s）后消失，此后普攻不再有额外伤害。
#[test]
fn camille_r_mark_expires() {
    let mut h = build_headless("camille_r_expire");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));

    h.cast_skill(3, Vec2::new(200.0, 0.0)).advance(0.2);
    assert!(has_r_mark(&h, enemy), "R 后应被标记");

    h.advance(1.8); // 总计 > 1.75s
    assert!(!has_r_mark(&h, enemy), "1.75s 后标记应消失");
    h.finish();
}
