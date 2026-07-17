#![cfg(test)]

//! Darius W 击杀返蓝减CD 测试。
//!
//! W 致残打击为强化普攻（攻击重置 + 额外伤害 + 减速），
//! 若击杀目标则返还 40 法力并减少 50% 冷却时间。
//!
//! 敌人默认 6000 HP，W bonus 32 物理伤害（Darius AD 64 × ratio 0.5），
//! 配合 5980 真伤（剩 20 HP）可由 W 斩杀。

use bevy::prelude::*;
use lol_core::attack::EventAttackEnd;
use lol_core::damage::{CommandDamageCreate, DamageType};
use lol_core::skill::CoolDown;

use super::tests::{build_headless, give_mana};

/// 手动触发一次普攻命中（仅触发 on-hit/被动，不造成基础 AA 伤害）。
fn attack_end(h: &mut crate::test_utils::ChampionTestHarness, target: Entity) {
    h.app.world_mut().trigger(EventAttackEnd {
        entity: h.champion,
        target,
    });
    h.app.update();
}

/// W 击杀目标后应返蓝 40。
#[test]
fn darius_w_refunds_mana_on_kill() {
    let mut h = build_headless("darius_w_mana_refund");
    give_mana(&mut h);

    // 让敌人残血（敌人 6000 HP，打 5980 剩 20，W bonus 32 斩杀）
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    h.app.world_mut().trigger(CommandDamageCreate {
        entity: enemy,
        source: h.champion,
        damage_type: DamageType::True,
        amount: 5980.0,
        tag: None,
    });
    h.advance(0.1);

    let mana_before = h.mana();
    h.cast_skill(1, Vec2::new(100.0, 0.0)); // W
    h.advance(0.1);

    // 触发普攻命中（不造成基础 AA 伤害，但触发 W on-hit 额外伤害）
    attack_end(&mut h, enemy);
    h.advance(0.1);

    // 确认敌人已死
    let enemy_hp = h.health(enemy);
    assert!(enemy_hp <= 0.0, "W 应斩杀残血敌人，实际 HP={enemy_hp:.1}");

    // 确认法力返还（W 扣 40，返还 40，回满）
    let mana_after = h.mana();
    assert!(
        mana_after > 995.0,
        "W 击杀应返蓝回满（after={mana_after:.1}）"
    );

    h.finish();
}

/// W 击杀目标后应减 CD 50%。
#[test]
fn darius_w_reduces_cooldown_on_kill() {
    let mut h = build_headless("darius_w_cd_reduce");
    give_mana(&mut h);

    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    h.app.world_mut().trigger(CommandDamageCreate {
        entity: enemy,
        source: h.champion,
        damage_type: DamageType::True,
        amount: 5980.0,
        tag: None,
    });
    h.advance(0.1);

    h.cast_skill(1, Vec2::new(100.0, 0.0)); // W
    h.advance(0.1);

    // 记下 W 冷却剩余时间
    let w_entity = h.skill_entity(1);
    let remaining_before = h
        .app
        .world()
        .get::<CoolDown>(w_entity)
        .unwrap()
        .timer
        .as_ref()
        .map(|t| t.remaining_secs())
        .unwrap_or(0.0);

    attack_end(&mut h, enemy);
    h.advance(0.1);

    // 记下减 CD 后的剩余时间
    let remaining_after = h
        .app
        .world()
        .get::<CoolDown>(w_entity)
        .unwrap()
        .timer
        .as_ref()
        .map(|t| t.remaining_secs())
        .unwrap_or(0.0);

    assert!(
        remaining_after < remaining_before,
        "W 击杀应减 CD 50%（before={remaining_before:.1}s, after={remaining_after:.1}s）"
    );
    assert!(
        (remaining_after - remaining_before * 0.5).abs() < 0.3,
        "W 减 CD 应为 50%（before={remaining_before:.1}s, 预期 half={:.1}s, 实际={remaining_after:.1}s）",
        remaining_before * 0.5,
    );

    h.finish();
}

/// W 若未击杀目标，不应返蓝或减 CD。
#[test]
fn darius_w_no_refund_without_kill() {
    let mut h = build_headless("darius_w_no_refund");
    give_mana(&mut h);

    // 满血敌人（1000 HP），W 无法斩杀
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    let mana_before = h.mana();
    let w_entity = h.skill_entity(1);
    let cd_before = h
        .app
        .world()
        .get::<CoolDown>(w_entity)
        .unwrap_or(&CoolDown {
            timer: None,
            duration: 5.0,
        })
        .timer
        .as_ref()
        .map(|t| t.remaining_secs());

    h.cast_skill(1, Vec2::new(100.0, 0.0)); // W
    h.advance(0.1);

    attack_end(&mut h, enemy);
    h.advance(0.1);

    let mana_after = h.mana();
    // W 扣 40 法力，未击杀不应返还，因此 mana_after = mana_before - 40
    let expected_mana = mana_before - 40.0;
    assert!(
        (mana_after - expected_mana).abs() < 0.5,
        "未击杀不应返蓝（before={mana_before:.1}, 应={expected_mana:.1}, 实际={mana_after:.1}）"
    );

    // CD 不应减少（即与之前相同或自然流逝）
    let cd_after = h
        .app
        .world()
        .get::<CoolDown>(w_entity)
        .unwrap()
        .timer
        .as_ref()
        .map(|t| t.remaining_secs());
    if let (Some(before), Some(after)) = (cd_before, cd_after) {
        assert!(
            after >= before - 0.5,
            "未击杀不应减 CD（before={before:.1}s, after={after:.1}s）"
        );
    }

    h.finish();
}