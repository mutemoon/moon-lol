#![cfg(test)]

//! Volibear Q（雷鸣重击）集成测试。
//!
//! Q：获得移速加成，下次普攻造成额外物理伤害（calculated_damage）并眩晕目标。
//! calculated_damage @1级 = base(0) + 1.0*AD + 1.6*AD = 2.6*AD = 2.6*65 = 169。

use bevy::math::{Vec2, Vec3};
use lol_core::base::buff::Buffs;
use lol_core::buffs::common_buffs::BuffMoveSpeed;

use super::tests::{attack_end, build_headless, is_stunned};

/// Q 强化的下次普攻应造成 169 额外物理伤害。
#[test]
fn volibear_q_empowers_next_attack_bonus_damage() {
    let mut h = build_headless("volibear_q_bonus");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    let hp_before = h.health(enemy);
    let mana_before = h.mana();

    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.1);
    attack_end(&mut h, enemy);
    h.advance(0.1);

    let dealt = hp_before - h.health(enemy);
    assert!(
        (dealt - 169.0).abs() < 1.5,
        "Q 强化普攻应造成 2.6*AD=169 额外物理伤害，实际 {dealt}"
    );
    assert!(
        h.mana() < mana_before,
        "Q 施放应消耗法力（冷却为 0.0 不检查 is_cooling）"
    );
    h.finish();
}

/// Q 强化的下次普攻应眩晕目标。
#[test]
fn volibear_q_stuns_target() {
    let mut h = build_headless("volibear_q_stun");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    h.cast_skill(0, Vec2::new(100.0, 0.0)).advance(0.1);
    attack_end(&mut h, enemy);
    h.advance(0.1);

    assert!(is_stunned(&h, enemy), "Q 强化普攻应眩晕目标 1s");
    h.finish();
}

/// Q 施放应给自身挂上移速加成 buff。
#[test]
fn volibear_q_grants_move_speed() {
    let mut h = build_headless("volibear_q_ms");

    h.cast_skill(0, Vec2::new(800.0, 0.0)).advance(0.1);

    let mut has_ms = false;
    if let Some(buffs) = h.app.world().get::<Buffs>(h.champion) {
        for buff in buffs.iter() {
            if h.app.world().get::<BuffMoveSpeed>(*buff).is_some() {
                has_ms = true;
                break;
            }
        }
    }
    assert!(has_ms, "Q 施放应给自身挂上移速加成");
    h.finish();
}
