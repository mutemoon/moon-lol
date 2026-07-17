#![cfg(test)]

use bevy::math::{Vec2, Vec3};
use lol_core::base::buff::Buffs;
use lol_core::buffs::cc_debuffs::DebuffSlow;

use super::tests::{build_headless, give_mana};

/// 锥形内敌人应被拉到 Darius 脚边，且 Darius 自身不位移。
#[test]
fn darius_e_pulls_enemies_in_cone_to_feet() {
    let mut h = build_headless("darius_e_pull");
    give_mana(&mut h);
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let darius_before = h.position(h.champion);
    let mana_before = h.mana();

    h.cast_skill(2, Vec2::new(500.0, 0.0)).advance(1.0);

    let enemy_after = h.position(enemy);
    let darius_after = h.position(h.champion);
    assert!(
        enemy_after.x.abs() < 30.0,
        "锥形内敌人应被拉到 Darius 脚边（x≈0），实际 x = {}",
        enemy_after.x
    );
    assert!(
        (darius_after - darius_before).length() < 5.0,
        "Darius 自身不应位移，实际移动 {}",
        (darius_after - darius_before).length()
    );
    assert!(
        !h.can_cast(2),
        "E 施放后应进入冷却"
    );
    assert!(h.mana() < mana_before, "E 施放应消耗法力");
    h.finish();
}

/// 锥形外（背后）的敌人不应被拉回。
#[test]
fn darius_e_does_not_pull_outside_cone() {
    let mut h = build_headless("darius_e_no_pull_outside");
    give_mana(&mut h);
    // 锥形朝 +X；敌人在 -X 方向 300 处（范围内但锥形外）
    let enemy = h.add_enemy(Vec3::new(-300.0, 0.0, 0.0));
    let before = h.position(enemy);

    h.cast_skill(2, Vec2::new(500.0, 0.0)).advance(1.0);

    let after = h.position(enemy);
    assert!(
        (after - before).length() < 5.0,
        "锥形外敌人不应被拉回，实际移动 {}",
        (after - before).length()
    );
    h.finish();
}

/// 被拉的敌人应挂 40% 减速 1 秒。
#[test]
fn darius_e_applies_slow_to_pulled() {
    let mut h = build_headless("darius_e_slow");
    give_mana(&mut h);
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    h.cast_skill(2, Vec2::new(500.0, 0.0)).advance(0.3);

    let buffs = h.app.world().get::<Buffs>(enemy);
    assert!(buffs.is_some(), "敌人应有 Buffs 组件");
    let mut found_slow = false;
    if let Some(buffs) = buffs {
        for buff_entity in buffs.iter() {
            if h.app.world().get::<DebuffSlow>(*buff_entity).is_some() {
                found_slow = true;
                break;
            }
        }
    }
    assert!(found_slow, "被拉敌人应挂 40% 减速");
    h.finish();
}
