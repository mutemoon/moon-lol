#![cfg(test)]

//! Camille W 集成测试（TDD）。
//!
//! W 为延迟蓄力锥形（扇形）伤害：以**施法者**为顶点，朝施法点方向，蓄力 0.75s 后
//! 造成物理伤害 + 减速。这是对 `AoEOrigin::Caster` + `Sector` + `delay_from_data_value`
//! 原语的组合验证：
//!
//! - 原点 = Caster（扇形顶点在施法者）
//! - 朝向 = 施法者 → 施法点（forward）
//! - 形状 = Sector{ radius=BlastLength=650, angle=ConeAngle=35 }
//! - 延迟 = dataValue ChargeDuration=0.75（蓄力）
//! - 伤害 = spell ron 的 `base_damage_total`（物理）
//! - 减速由 `on_camille_damage_hit` observer 在伤害结算时施加

use bevy::math::{Vec2, Vec3};
use lol_core::base::buff::Buffs;
use lol_core::buffs::cc_debuffs::DebuffSlow;

use super::tests::build_headless;

/// 延迟生效：ChargeDuration 0.75s 前无伤害，之后前方敌人受伤。
#[test]
fn camille_w_delayed() {
    let mut h = build_headless("camille_w_delayed");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let hp_before = h.health(enemy);
    let mana_before = h.mana();

    h.cast_skill(1, Vec2::new(650.0, 0.0)).advance(0.5);
    assert!(
        (h.health(enemy) - hp_before).abs() < 0.01,
        "蓄力 0.75s 结束前不应造成伤害"
    );

    h.advance(0.5); // 总 1.0s > 0.75
    let dealt = hp_before - h.health(enemy);
    assert!(dealt > 0.0, "蓄力结束后前方敌人应受到物理伤害");
    assert!(!h.can_cast(1), "W 施放后应进入冷却");
    assert!(h.mana() < mana_before, "W 施放应消耗法力");
    h.finish();
}

/// 半径边界：BlastLength 650 内命中，超出不命中。
#[test]
fn camille_w_outside_radius() {
    let mut h = build_headless("camille_w_radius");
    let inside = h.add_enemy(Vec3::new(600.0, 0.0, 0.0));
    let outside = h.add_enemy(Vec3::new(700.0, 0.0, 0.0));
    let hp_in = h.health(inside);
    let hp_out = h.health(outside);

    h.cast_skill(1, Vec2::new(650.0, 0.0)).advance(1.0);

    assert!(h.health(inside) < hp_in, "650 内（600）应受伤");
    assert!(
        (h.health(outside) - hp_out).abs() < 0.01,
        "650 外（700）不应受伤"
    );
    h.finish();
}

/// 朝向验证：施法者背后的敌人不在扇形内（forward 指向施法点）。
#[test]
fn camille_w_behind_caster_unharmed() {
    let mut h = build_headless("camille_w_behind");
    let enemy = h.add_enemy(Vec3::new(-200.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    // 朝 +X 施法，敌人位于 -X（背后）
    h.cast_skill(1, Vec2::new(650.0, 0.0)).advance(1.0);

    assert!(
        (h.health(enemy) - hp_before).abs() < 0.01,
        "施法者背后的敌人不应受伤（扇形朝向施法点方向）"
    );
    h.finish();
}

/// 扇形角度：ConeAngle 35（半角 17.5°）。角内命中，角外不命中。
#[test]
fn camille_w_cone_angle() {
    let mut h = build_headless("camille_w_cone");
    // 距离 300，半角 17.5° → 侧向阈值 ≈ 300*tan(17.5°) ≈ 94
    let in_cone = h.add_enemy(Vec3::new(300.0, 0.0, 50.0)); // ≈9.5° < 17.5
    let out_cone = h.add_enemy(Vec3::new(300.0, 0.0, 200.0)); // ≈33.7° > 17.5
    let hp_in = h.health(in_cone);
    let hp_out = h.health(out_cone);

    h.cast_skill(1, Vec2::new(650.0, 0.0)).advance(1.0);

    assert!(h.health(in_cone) < hp_in, "扇形内（≈9.5°）应受伤");
    assert!(
        (h.health(out_cone) - hp_out).abs() < 0.01,
        "扇形外（≈33.7°）不应受伤"
    );
    h.finish();
}

/// 延迟伤害仍触发命中 observer（减速）。
#[test]
fn camille_w_slow_applied() {
    let mut h = build_headless("camille_w_slow");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    h.cast_skill(1, Vec2::new(650.0, 0.0)).advance(1.0);

    // DebuffSlow 经 BuffOf 关联挂在 buff 子实体上，需遍历敌人的 Buffs 查找
    let mut has_slow = false;
    if let Some(buffs) = h.app.world().get::<Buffs>(enemy) {
        for buff in buffs.iter() {
            if h.app.world().get::<DebuffSlow>(*buff).is_some() {
                has_slow = true;
                break;
            }
        }
    }
    assert!(has_slow, "延迟伤害应触发 on_camille_damage_hit 施加减速");
    h.finish();
}
