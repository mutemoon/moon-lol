#![cfg(test)]

//! Camille E（钩索 / Hookshot）两段式集成测试。
//!
//! - E1：发射粘性飞弹，碰墙后挂墙壁锚点并拉向墙壁。
//! - E2：从墙壁朝目标方向冲刺，命中英雄附加眩晕/伤害/减速 + 攻速加成。

use bevy::math::Vec2;
use bevy::prelude::*;
use lol_base::spell::Spell;
use lol_core::attack::BuffAttack;
use lol_core::base::buff::Buffs;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::missile::EventMissileHit;
use lol_core::skill::Skill;

use super::buffs::BuffCamilleWallCling;
use super::tests::build_headless;

/// 获取 Camille E 技能的 Spell Handle（用于模拟飞弹命中）。
fn e_spell_handle(h: &crate::test_utils::ChampionTestHarness) -> Handle<Spell> {
    let skill_entity = h.skill_entity(2);
    h.app.world().get::<Skill>(skill_entity).unwrap().spell.clone()
}

/// 模拟 E 粘性飞弹命中墙壁，触发 `EventMissileHit`。
fn trigger_e_missile_hit(h: &mut crate::test_utils::ChampionTestHarness, point: Vec3) {
    let spell = e_spell_handle(h);
    h.app.world_mut()
        .entity_mut(h.champion)
        .trigger(|e| EventMissileHit {
            source: e,
            spell,
            point,
        });
}

/// 辅助断言：敌方实体是否存在 DebuffStun（在子 buff 实体上）。
fn has_stun(h: &crate::test_utils::ChampionTestHarness, entity: Entity) -> bool {
    h.app
        .world()
        .get::<Buffs>(entity)
        .map_or(false, |b| b.iter().any(|e| h.app.world().get::<DebuffStun>(e).is_some()))
}

// ── E1 阶段 ──

/// E1 施放后应设置重施窗口（Stage 2）。
#[test]
fn camille_e1_sets_recast_window() {
    let mut h = build_headless("camille_e1_window");
    h.cast_skill(2, Vec2::new(100.0, 0.0)).advance(0.1);

    let skill_e = h.skill_entity(2);
    assert!(
        h.has_recast_window(skill_e),
        "E1 后应存在重施窗口"
    );
    assert_eq!(
        h.recast_window_stage(skill_e),
        Some(2),
        "E1 后重施窗口 stage 应为 2"
    );
    h.finish();
}

/// E1 飞弹碰墙 → `EventMissileHit` → 应挂载墙壁锚点。
#[test]
fn camille_e1_missile_hit_creates_wall_cling() {
    let mut h = build_headless("camille_e1_cling");
    // 先施放 E1
    h.cast_skill(2, Vec2::new(100.0, 0.0)).advance(0.1);

    // 模拟飞弹碰墙
    trigger_e_missile_hit(&mut h, Vec3::new(200.0, 0.0, 0.0));
    h.advance(0.1);

    // 检查墙壁锚点 BuffCamilleWallCling
    let has_cling = h
        .app
        .world_mut()
        .query::<&BuffCamilleWallCling>()
        .iter(h.app.world())
        .next()
        .is_some();
    assert!(has_cling, "飞弹碰墙后应存在 BuffCamilleWallCling");

    // 检查朝向墙壁移动（冠军初始位置 (0,0,0)，墙壁点 (200,0,0)）
    let pos = h.position(h.champion);
    assert!(
        pos.x > 1.0,
        "应开始向墙壁冲刺，实际 x = {:.1}",
        pos.x
    );
    h.finish();
}

// ── E2 阶段 ──

/// E2 命中路径上的敌人 → 眩晕 + 攻速加成。
#[test]
fn camille_e2_stuns_enemy_in_path() {
    let mut h = build_headless("camille_e2_stun");
    // 施放 E1
    h.cast_skill(2, Vec2::new(300.0, 0.0)).advance(0.1);
    // 模拟飞弹碰墙（墙壁在左边）
    trigger_e_missile_hit(&mut h, Vec3::new(50.0, 0.0, 0.0));
    h.advance(0.3); // 等冲到墙边

    // 在冲刺路径上放置敌人（位于冠军→目标方向）
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    // E2 向目标方向冲刺
    h.cast_skill(2, Vec2::new(500.0, 0.0)).advance(0.5);

    // 敌人应被眩晕
    assert!(
        has_stun(&h, enemy),
        "E2 路径上的敌人应被眩晕"
    );

    // 冠军应有攻速加成
    let as_bonus = h
        .app
        .world()
        .get::<BuffAttack>(h.champion)
        .expect("E2 后 Camille 应有 BuffAttack")
        .bonus_attack_speed;
    assert!(
        (as_bonus - 0.35).abs() < 1e-3,
        "E2 应赋予 0.35 攻速，实际 {:.3}",
        as_bonus
    );

    assert!(!h.can_cast(2), "E2 施放后应进入冷却");
    h.finish();
}

/// E2 无敌人时 → 冲刺 + 攻速加成，无眩晕。
#[test]
fn camille_e2_no_enemy_dashes_without_stun() {
    let mut h = build_headless("camille_e2_noenemy");
    h.cast_skill(2, Vec2::new(100.0, 0.0)).advance(0.1);
    trigger_e_missile_hit(&mut h, Vec3::new(50.0, 0.0, 0.0));
    h.advance(0.3);

    // 不放敌人，E2 应正常冲刺但无眩晕
    h.cast_skill(2, Vec2::new(500.0, 0.0)).advance(0.5);

    // 冠军位置应发生变化（已冲刺）
    let pos = h.position(h.champion);
    assert!(
        pos.x > 10.0,
        "E2 无敌人时也应冲刺，实际 x = {:.1}",
        pos.x
    );

    // 攻速加成应生效
    let as_bonus = h
        .app
        .world()
        .get::<BuffAttack>(h.champion)
        .expect("E2 后 Camille 应有 BuffAttack")
        .bonus_attack_speed;
    assert!(
        (as_bonus - 0.35).abs() < 1e-3,
        "E2 应赋予 0.35 攻速，实际 {:.3}",
        as_bonus
    );

    assert!(!h.can_cast(2), "E2 施放后应进入冷却");
    h.finish();
}

/// E2 攻速加成应在 ASDuration（5s）后消失。
#[test]
fn camille_e_as_expires() {
    let mut h = build_headless("camille_e_expire");
    h.cast_skill(2, Vec2::new(100.0, 0.0)).advance(0.1);
    trigger_e_missile_hit(&mut h, Vec3::new(50.0, 0.0, 0.0));
    h.advance(0.3);
    h.cast_skill(2, Vec2::new(500.0, 0.0)).advance(0.2);

    assert!(
        h.app.world().get::<BuffAttack>(h.champion).is_some(),
        "E2 后应存在攻速加成"
    );

    h.advance(5.3); // 总计 > 5s
    assert!(
        h.app.world().get::<BuffAttack>(h.champion).is_none(),
        "5s 后攻速加成应消失"
    );
    h.finish();
}