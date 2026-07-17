#![cfg(test)]

//! Volibear 被动（风暴之力）集成测试。
//!
//! 被动：普攻命中叠加层数（上限 5），每层 +5% 攻速；脱战 6s 清零。

use bevy::math::Vec3;

use super::tests::{attack_end, attack_speed_bonus, build_headless};
use crate::volibear::passive::VOLIBEAR_P_ATTACK_SPEED_PER_STACK;

/// 普攻命中一次应获得 1 层（+5% 攻速）。
#[test]
fn volibear_p_grants_attack_speed_stack() {
    let mut h = build_headless("volibear_p_stack");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    attack_end(&mut h, enemy);

    let bonus = attack_speed_bonus(&h);
    assert!(
        (bonus - VOLIBEAR_P_ATTACK_SPEED_PER_STACK).abs() < 0.001,
        "普攻一次应获得 {} 攻速加成，实际 {bonus}",
        VOLIBEAR_P_ATTACK_SPEED_PER_STACK,
    );
    h.finish();
}

/// 连续普攻 6 次，层数应封顶在 5 层（+25% 攻速）。
#[test]
fn volibear_p_stacks_cap_at_five() {
    let mut h = build_headless("volibear_p_cap");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    for _ in 0..6 {
        attack_end(&mut h, enemy);
    }

    let bonus = attack_speed_bonus(&h);
    assert!(
        (bonus - 5.0 * VOLIBEAR_P_ATTACK_SPEED_PER_STACK).abs() < 0.001,
        "6 次普攻应封顶 {} 攻速，实际 {bonus}",
        5.0 * VOLIBEAR_P_ATTACK_SPEED_PER_STACK,
    );
    h.finish();
}

/// 脱战 6s 后层数应清零。
#[test]
fn volibear_p_decays_out_of_combat() {
    let mut h = build_headless("volibear_p_decay");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    attack_end(&mut h, enemy);
    assert!((attack_speed_bonus(&h) - VOLIBEAR_P_ATTACK_SPEED_PER_STACK).abs() < 0.001);

    h.advance(6.2); // > 6s 持续时间

    let bonus = attack_speed_bonus(&h);
    assert!(
        bonus.abs() < 0.001,
        "脱战 6s 后攻速加成应清零，实际 {bonus}"
    );
    h.finish();
}
