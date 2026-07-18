#![cfg(test)]

use bevy::math::Vec2;
use bevy::prelude::*;
use lol_core::buffs::cc_debuffs::DebuffSlow;

use crate::irelia::buffs::DebuffIreliaUnsteady;
use crate::irelia::tests::*;

/// R 命中：造成 missile_damage 魔法伤害 + 不稳标记 + 减速。
#[test]
fn irelia_r_damages_marks_and_slows() {
    let mut h = build_headless("irelia_r_hit");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let mana_before = h.mana();

    let ad = ad(&h);
    let expected = h
        .get_skill_value(
            3,
            "missile_damage",
            1,
            |stat| if stat == 2 { ad } else { 0.0 },
        )
        .unwrap();

    h.cast_skill(3, Vec2::new(300.0, 0.0));
    h.advance(0.2);

    assert!(
        (h.health(enemy) - (6000.0 - expected)).abs() < 1.0,
        "R 应造成 {expected} 伤害，实际剩余 {}",
        h.health(enemy)
    );
    assert!(
        has_buff::<DebuffIreliaUnsteady>(&h, enemy),
        "R 应标记敌人不稳"
    );
    assert!(
        has_buff::<DebuffSlow>(&h, enemy),
        "R 应减速敌人（DebuffSlow）"
    );
    assert!(!h.can_cast(3), "R 施放后应进入冷却");
    assert!(h.mana() < mana_before, "R 施放应消耗法力");
    h.finish();
}

/// R 不命中队友（同队过滤）。
#[test]
fn irelia_r_ignores_allies() {
    let mut h = build_headless("irelia_r_ally");
    let ally = h.add_ally(Vec3::new(300.0, 0.0, 0.0));

    h.cast_skill(3, Vec2::new(300.0, 0.0));
    h.advance(0.2);

    assert!((h.health(ally) - 6000.0).abs() < 0.01, "R 不应伤害同队盟友");
    h.finish();
}
