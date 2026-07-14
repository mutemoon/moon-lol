#![cfg(test)]

//! Camille E（钩索 / Hookshot）攻速加成集成测试（TDD）。
//!
//! E 二段（E2）命中后获得攻速加成（`ASBuff`，1 级 = 0.35），持续 `ASDuration`（5s）。
//! 地形钩索按位移框架 Phase 4.2 暂缓，此处验证可独立核验的攻速部分。

use bevy::math::Vec2;
use lol_core::attack::BuffAttack;

use super::tests::build_headless;

/// E2 应按等级赋予攻速（1 级 = 0.35）。
#[test]
fn camille_e2_grants_attack_speed() {
    let mut h = build_headless("camille_e_as");
    // E1 开启重施窗口，E2 命中后挂攻速
    h.cast_skill(2, Vec2::new(100.0, 0.0)).advance(0.2);
    h.cast_skill(2, Vec2::new(100.0, 0.0)).advance(0.2);

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
    h.finish();
}

/// 攻速加成应在 ASDuration（5s）后消失。
#[test]
fn camille_e_as_expires() {
    let mut h = build_headless("camille_e_expire");
    h.cast_skill(2, Vec2::new(100.0, 0.0)).advance(0.2);
    h.cast_skill(2, Vec2::new(100.0, 0.0)).advance(0.2);
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
