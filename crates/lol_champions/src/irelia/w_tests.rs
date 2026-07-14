#![cfg(test)]

use bevy::math::Vec2;
use bevy::prelude::*;

use crate::irelia::tests::*;

/// W1 进入防御姿态，减免 50% 受到伤害。
#[test]
fn irelia_w1_grants_50_percent_damage_reduction() {
    let mut h = build_headless("irelia_w_dr");
    let enemy = h.add_enemy(Vec3::new(500.0, 0.0, 0.0));
    let max_hp = h.health(h.champion);

    h.cast_skill(1, Vec2::new(0.0, 0.0));
    h.advance(0.05);
    // 魔法伤害绕过护甲，单独验证 50% 减伤
    apply_magic_damage(&mut h, enemy, 100.0);
    h.advance(0.1);

    assert!(
        (h.health(h.champion) - (max_hp - 50.0)).abs() < 1.0,
        "W1 减伤 50%：100 伤害应只扣 50，实际 {}",
        h.health(h.champion)
    );
    h.finish();
}

/// W2 释放伤害随蓄力时长线性增长：满蓄（≥0.75s）达到 max_damage_calc。
#[test]
fn irelia_w2_full_charge_deals_max_damage() {
    let mut h = build_headless("irelia_w_max");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    let ad = ad(&h);
    let getter = |stat: u8| if stat == 2 { ad } else { 0.0 };
    let max_dmg = h.get_skill_value(1, "max_damage_calc", 1, getter).unwrap();

    h.cast_skill(1, Vec2::new(300.0, 0.0));
    h.advance(0.8);
    h.cast_skill(1, Vec2::new(300.0, 0.0));
    h.advance(0.2);

    assert!(
        (h.health(enemy) - (6000.0 - max_dmg)).abs() < 1.0,
        "满蓄 W2 应造成 max_damage_calc={max_dmg}，实际剩余 {}",
        h.health(enemy)
    );
    h.finish();
}

/// W2 蓄力越短伤害越低：短蓄伤害应小于满蓄伤害。
#[test]
fn irelia_w2_short_charge_deals_less_than_full() {
    let mut h = build_headless("irelia_w_short");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    // 满蓄
    h.cast_skill(1, Vec2::new(300.0, 0.0));
    h.advance(0.8);
    h.cast_skill(1, Vec2::new(300.0, 0.0));
    h.advance(0.2);
    let full_hp = h.health(enemy);

    // 重新构建一个干净环境做短蓄对比
    let mut h2 = build_headless("irelia_w_short2");
    let enemy2 = h2.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    h2.cast_skill(1, Vec2::new(300.0, 0.0));
    h2.advance(0.05);
    h2.cast_skill(1, Vec2::new(300.0, 0.0));
    h2.advance(0.2);
    let short_hp = h2.health(enemy2);

    assert!(
        short_hp > full_hp,
        "短蓄伤害应低于满蓄：短蓄剩余 {short_hp} 应大于满蓄剩余 {full_hp}"
    );
    h.finish();
    h2.finish();
}

/// W2 释放后清除减伤：再次受击为全额伤害。
#[test]
fn irelia_w2_removes_damage_reduction() {
    let mut h = build_headless("irelia_w_remove_dr");
    let enemy = h.add_enemy(Vec3::new(500.0, 0.0, 0.0));
    let max_hp = h.health(h.champion);

    h.cast_skill(1, Vec2::new(0.0, 0.0));
    h.advance(0.1);
    h.cast_skill(1, Vec2::new(0.0, 0.0));
    h.advance(0.1);
    // W2 释放后减伤已清除；魔法伤害绕过护甲，全额 100
    apply_magic_damage(&mut h, enemy, 100.0);
    h.advance(0.1);

    assert!(
        (h.health(h.champion) - (max_hp - 100.0)).abs() < 1.0,
        "W2 释放后应无减伤：100 伤害全额扣 100，实际 {}",
        h.health(h.champion)
    );
    h.finish();
}

/// W1 蓄力超过最大时长后自动到期，减伤随之回收。
#[test]
fn irelia_w_expires_after_max_duration() {
    let mut h = build_headless("irelia_w_expire");
    let enemy = h.add_enemy(Vec3::new(500.0, 0.0, 0.0));
    let max_hp = h.health(h.champion);

    h.cast_skill(1, Vec2::new(0.0, 0.0));
    h.advance(1.6); // > 1.5s 最大蓄力时长
    // 蓄力到期后减伤已回收；魔法伤害绕过护甲，全额 100
    apply_magic_damage(&mut h, enemy, 100.0);
    h.advance(0.1);

    assert!(
        (h.health(h.champion) - (max_hp - 100.0)).abs() < 1.0,
        "蓄力到期后应无减伤：100 伤害全额扣 100，实际 {}",
        h.health(h.champion)
    );
    h.finish();
}
