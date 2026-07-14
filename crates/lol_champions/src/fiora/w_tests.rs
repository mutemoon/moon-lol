#![cfg(test)]

//! Fiora W（斗转刺 / Riposte）单元测试。
//!
//! W 机制：0.75s 招架期免疫所有非真实伤害与控制；招架结束向前方矩形刺出，
//! 对首个敌方英雄造成魔法伤害。若招架期间被硬控命中，反刺改为眩晕，否则减速。

use bevy::math::{Vec2, Vec3};
use lol_core::base::buff::{BuffOf, Buffs};
use lol_core::buffs::cc_debuffs::{DebuffSlow, DebuffStun};
use lol_core::damage::{CommandDamageCreate, DamageType};

use crate::fiora::tests::build_headless;
use crate::fiora::w::BuffFioraW;
use crate::test_utils::ChampionTestHarness;

const EPSILON: f32 = 1e-3;

/// 招架期间承受物理伤害应被 100% 减免（无生命值损失）。
#[test]
fn fiora_w_parry_negates_physical_damage() {
    let mut h = build_headless("fiora_w_parry_physical");
    let enemy = h.add_enemy(Vec3::new(-200.0, 0.0, 0.0)); // 远处敌人作为伤害来源
    let caster = h.champion;
    let hp_before = h.health(caster);

    h.cast_skill(1, Vec2::new(400.0, 0.0)).advance(0.1);
    assert!(fiora_w_active(&h), "招架期内应存在 BuffFioraW");

    h.apply_damage(enemy, 100.0).advance(0.1);

    let hp_loss = hp_before - h.health(caster);
    assert!(
        hp_loss < EPSILON,
        "招架应完全减免物理伤害，实际损失 {:.1}",
        hp_loss
    );
    h.finish();
}

/// 真实伤害无视招架减免（True 分支跳过所有防御）。
#[test]
fn fiora_w_true_damage_penetrates_parry() {
    let mut h = build_headless("fiora_w_true_damage");
    let enemy = h.add_enemy(Vec3::new(-200.0, 0.0, 0.0));
    let caster = h.champion;
    let hp_before = h.health(caster);

    h.cast_skill(1, Vec2::new(400.0, 0.0)).advance(0.1);

    h.app.world_mut().trigger(CommandDamageCreate {
        entity: caster,
        source: enemy,
        damage_type: DamageType::True,
        amount: 100.0,
        tag: None,
    });
    h.advance(0.1);

    let hp_loss = hp_before - h.health(caster);
    assert!(
        (hp_loss - 100.0).abs() < 1.0,
        "真实伤害应穿透招架，实际损失 {:.1}",
        hp_loss
    );
    h.finish();
}

/// 招架期间被硬控命中：CC 不沾身（被免疫销毁），并记录 parried_hard_cc。
#[test]
fn fiora_w_parry_blocks_hard_cc() {
    let mut h = build_headless("fiora_w_parry_cc");
    let caster = h.champion;

    h.cast_skill(1, Vec2::new(400.0, 0.0)).advance(0.1);

    // 施加眩晕指向菲奥娜：应被 ImmuneToCC 立即销毁
    h.app
        .world_mut()
        .entity_mut(caster)
        .with_related::<BuffOf>(DebuffStun::new(1.0));
    h.advance(0.1);

    assert!(
        !has_buff_component::<DebuffStun>(&h, caster),
        "招架期内眩晕应被免疫销毁"
    );
    assert_eq!(
        fiora_w_parried_hard_cc(&h),
        Some(true),
        "招架硬控应记录 parried_hard_cc"
    );

    // 招架结束后仍不应残留眩晕
    h.advance(0.9);
    assert!(
        !has_buff_component::<DebuffStun>(&h, caster),
        "招架结束后不应残留眩晕"
    );
    assert!(!fiora_w_active(&h), "招架结束后 BuffFioraW 应被移除");
    h.finish();
}

/// 反刺对矩形内首个敌方英雄造成魔法伤害（1 级 BaseDamage = 70）。
#[test]
fn fiora_w_counter_thrust_deals_magic_damage() {
    let mut h = build_headless("fiora_w_thrust_damage");
    let enemy = h.add_enemy(Vec3::new(400.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    // 朝敌人方向施放 W，推进过 0.75s 招架期触发反刺
    h.cast_skill(1, Vec2::new(400.0, 0.0)).advance(0.9);

    let hp_loss = hp_before - h.health(enemy);
    assert!(
        (hp_loss - 70.0).abs() < 1.0,
        "反刺应造成 70 点魔法伤害（1 级），实际损失 {:.1}",
        hp_loss
    );
    h.finish();
}

/// 未招架硬控时，反刺施加减速（|MSSlowPercent| = 0.5，持续 2s）。
#[test]
fn fiora_w_counter_thrust_slows_when_no_cc_parried() {
    let mut h = build_headless("fiora_w_thrust_slow");
    let enemy = h.add_enemy(Vec3::new(400.0, 0.0, 0.0));

    h.cast_skill(1, Vec2::new(400.0, 0.0)).advance(0.9);

    let slow = enemy_debuff_slow(&h, enemy);
    assert!(slow.is_some(), "反刺应施加减速");
    assert!(
        (slow.unwrap() - 0.5).abs() < EPSILON,
        "减速比例应为 0.5，实际 {:.2}",
        slow.unwrap()
    );
    assert!(
        !has_buff_component::<DebuffStun>(&h, enemy),
        "未招架硬控时不应眩晕"
    );
    h.finish();
}

/// 招架硬控后，反刺改为眩晕（CCDuration = 2s）而非减速。
#[test]
fn fiora_w_counter_thrust_stuns_when_hard_cc_parried() {
    let mut h = build_headless("fiora_w_thrust_stun");
    let caster = h.champion;
    let enemy = h.add_enemy(Vec3::new(400.0, 0.0, 0.0));

    // 施放 W 并招架一次硬控
    h.cast_skill(1, Vec2::new(400.0, 0.0)).advance(0.1);
    h.app
        .world_mut()
        .entity_mut(caster)
        .with_related::<BuffOf>(DebuffStun::new(1.0));
    h.advance(0.1);

    // 推进过招架期触发反刺
    h.advance(0.8);

    assert!(
        has_buff_component::<DebuffStun>(&h, enemy),
        "招架硬控后反刺应眩晕敌人"
    );
    assert!(
        !has_buff_component::<DebuffSlow>(&h, enemy),
        "招架硬控后反刺不应减速"
    );
    h.finish();
}

// ── helpers ──

fn fiora_w_active(h: &ChampionTestHarness) -> bool {
    let world = h.app.world();
    let Some(buffs) = world.get::<Buffs>(h.champion) else {
        return false;
    };
    for buff_entity in buffs.iter() {
        if world.get::<BuffFioraW>(*buff_entity).is_some() {
            return true;
        }
    }
    false
}

fn fiora_w_parried_hard_cc(h: &ChampionTestHarness) -> Option<bool> {
    let world = h.app.world();
    let buffs = world.get::<Buffs>(h.champion)?;
    for buff_entity in buffs.iter() {
        if let Some(w) = world.get::<BuffFioraW>(*buff_entity) {
            return Some(w.parried_hard_cc);
        }
    }
    None
}

fn has_buff_component<T: bevy::prelude::Component>(
    h: &ChampionTestHarness,
    entity: bevy::prelude::Entity,
) -> bool {
    let world = h.app.world();
    let Some(buffs) = world.get::<Buffs>(entity) else {
        return false;
    };
    for buff_entity in buffs.iter() {
        if world.get::<T>(*buff_entity).is_some() {
            return true;
        }
    }
    false
}

fn enemy_debuff_slow(h: &ChampionTestHarness, enemy: bevy::prelude::Entity) -> Option<f32> {
    let world = h.app.world();
    let buffs = world.get::<Buffs>(enemy)?;
    for buff_entity in buffs.iter() {
        if let Some(slow) = world.get::<DebuffSlow>(*buff_entity) {
            return Some(slow.percent);
        }
    }
    None
}
