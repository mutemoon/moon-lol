#![cfg(test)]

use bevy::math::{Vec2, Vec3};

use crate::darius::tests::*;

/// 出血应叠加到单层 buff 上，最多 5 层（而非每次命中生成新 buff）。
#[test]
fn darius_bleed_stacks_up_to_five() {
    let mut h = build_headless("darius_bleed_stack");
    give_mana(&mut h);
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));

    // 命中 5 次，每次少量推进以处理事件，但不足以触发 DoT 周期（1.26s）
    for _ in 0..5 {
        darius_hit(&mut h, enemy, 10.0);
        h.advance(0.01);
    }

    assert_eq!(
        bleed_stacks(&h, enemy),
        Some(5),
        "5 次命中应将出血叠满 5 层"
    );
    h.finish();
}

/// 出血每周期造成 0.3*AD 物理伤害（每层），敌人护甲为 0 故全额生效。
#[test]
fn darius_bleed_deals_dot_per_tick() {
    let mut h = build_headless("darius_bleed_dot");
    give_mana(&mut h);
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let ad = darius_ad(&h);

    // 1 层出血
    darius_hit(&mut h, enemy, 10.0);
    h.advance(0.01);
    let hp_after_hit = h.health(enemy);
    assert_eq!(bleed_stacks(&h, enemy), Some(1));

    // 推进超过一个周期（1.26s）-> 一次 DoT 结算
    h.advance(1.3);
    let hp_after_dot = h.health(enemy);

    let expected = 0.3 * ad; // bleed_damage_per_stack = 0.3 * AD，1 层
    assert!(
        (hp_after_hit - hp_after_dot - expected).abs() < 1.0,
        "1 层出血每周期应造成 0.3*AD={expected}（AD={ad}），实际 {}",
        hp_after_hit - hp_after_dot
    );
    h.finish();
}

/// DoT 结算的伤害不应再次叠加出血层数。
#[test]
fn darius_bleed_dot_does_not_restack() {
    let mut h = build_headless("darius_bleed_no_restack");
    give_mana(&mut h);
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));

    darius_hit(&mut h, enemy, 10.0);
    h.advance(0.01);
    assert_eq!(bleed_stacks(&h, enemy), Some(1));

    h.advance(1.3); // 触发一次 DoT
    assert_eq!(
        bleed_stacks(&h, enemy),
        Some(1),
        "DoT 伤害不应再次叠加出血层数"
    );
    h.finish();
}

/// 叠满 5 层出血时 Darius 获得诺克萨斯之力：+50% AD。
#[test]
fn darius_noxian_might_at_five_stacks() {
    let mut h = build_headless("darius_might");
    give_mana(&mut h);
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let base_ad = darius_ad(&h);

    for _ in 0..5 {
        darius_hit(&mut h, enemy, 10.0);
        h.advance(0.01);
    }

    assert!(has_might(&h), "5 层出血应触发诺克萨斯之力");
    let might_ad = darius_ad(&h);
    assert!(
        (might_ad - base_ad * 1.5).abs() < 1.0,
        "诺克萨斯之力应提供 +50% AD：{base_ad} -> {:.1}（期望 {:.1}）",
        might_ad,
        base_ad * 1.5
    );
    h.finish();
}

/// 诺克萨斯之力到期后 AD 恢复原值。
#[test]
fn darius_noxian_might_expires() {
    let mut h = build_headless("darius_might_expire");
    give_mana(&mut h);
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let base_ad = darius_ad(&h);

    for _ in 0..5 {
        darius_hit(&mut h, enemy, 10.0);
        h.advance(0.01);
    }
    assert!(has_might(&h));

    // 推进超过血怒持续时间
    h.advance(6.0);

    assert!(!has_might(&h), "血怒到期后应移除");
    assert!(
        (darius_ad(&h) - base_ad).abs() < 1.0,
        "血怒到期后 AD 应恢复原值 {}，实际 {}",
        base_ad,
        darius_ad(&h)
    );
    h.finish();
}

/// 出血持续 5 秒后自动消失。
#[test]
fn darius_bleed_expires_after_duration() {
    let mut h = build_headless("darius_bleed_expire");
    give_mana(&mut h);
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));

    darius_hit(&mut h, enemy, 10.0);
    h.advance(0.01);
    assert_eq!(bleed_stacks(&h, enemy), Some(1));

    h.advance(5.5); // > 5s 持续时间
    assert_eq!(bleed_stacks(&h, enemy), None, "5 秒后出血应过期清除");
    h.finish();
}

/// R 斩杀伤害随目标出血层数线性增长（每层 +30%）。
#[test]
fn darius_r_scales_with_hemorrhage_stacks() {
    let mut h = build_headless("darius_r_scaling");
    give_mana(&mut h);

    // 无出血目标
    let enemy_a = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.5);
    let dmg_no_stacks = 6000.0 - h.health(enemy_a);

    // 5 层出血目标
    let mut h2 = build_headless("darius_r_scaling_stacks");
    give_mana(&mut h2);
    let enemy_b = h2.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    for _ in 0..5 {
        darius_hit(&mut h2, enemy_b, 1.0);
        h2.advance(0.01);
    }
    assert_eq!(bleed_stacks(&h2, enemy_b), Some(5));
    h2.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.5);
    let dmg_five_stacks = 6000.0 - h2.health(enemy_b);

    assert!(
        dmg_five_stacks > dmg_no_stacks,
        "5 层出血应使 R 伤害高于无出血：5层={dmg_five_stacks} 无层={dmg_no_stacks}"
    );
    h.finish();
    h2.finish();
}
