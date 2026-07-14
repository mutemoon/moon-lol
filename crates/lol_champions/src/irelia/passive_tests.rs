#![cfg(test)]

use bevy::math::Vec2;
use bevy::prelude::*;
use lol_core::attack::{BuffAttack, EventAttackEnd};

use crate::irelia::tests::*;

/// 施放技能叠加被动：1 次施法 -> 1 层热诚 + 8% 攻速。
#[test]
fn irelia_passive_stacks_on_cast() {
    let mut h = build_headless("irelia_passive_stack");
    // 敌人放在 Q 范围外，避免伤害干扰；被动由 EventSkillCast 驱动，与命中无关
    let _ = h.add_enemy(Vec3::new(2000.0, 0.0, 0.0));

    h.cast_skill(0, Vec2::new(0.0, 0.0));
    h.advance(0.1);

    assert_eq!(fervor_charges(&h), Some(1), "一次施法应叠加 1 层热诚");

    let as_buff = h.app.world().get::<BuffAttack>(h.champion);
    assert!(as_buff.is_some(), "应有 BuffAttack 攻速加成");
    assert!(
        (as_buff.unwrap().bonus_attack_speed - 0.08).abs() < 1e-4,
        "1 层应提供 8% 攻速"
    );
    h.finish();
}

/// 满层（4 层）时普攻命中附带 20% AD 额外魔法伤害。
#[test]
fn irelia_passive_max_stacks_bonus_on_hit() {
    let mut h = build_headless("irelia_passive_max");
    // 敌人放在所有技能范围外，确保 4 次施法不对其造成伤害
    let enemy = h.add_enemy(Vec3::new(2000.0, 0.0, 0.0));

    // 4 次施法（Q/W1/E1/R）叠满 4 层；全部朝原点施放，不触及敌人
    h.cast_skill(0, Vec2::new(0.0, 0.0));
    h.advance(0.05);
    h.cast_skill(1, Vec2::new(0.0, 0.0));
    h.advance(0.05);
    h.cast_skill(2, Vec2::new(0.0, 0.0));
    h.advance(0.05);
    h.cast_skill(3, Vec2::new(0.0, 0.0));
    h.advance(0.05);

    assert_eq!(fervor_charges(&h), Some(4), "4 次施法应叠满 4 层");
    assert!(
        (h.health(enemy) - 6000.0).abs() < 0.01,
        "施法本身不应伤害远处敌人"
    );

    // 满层普攻命中 -> 20% AD 额外魔法伤害
    let ad = ad(&h);
    h.app.world_mut().trigger(EventAttackEnd {
        entity: h.champion,
        target: enemy,
    });
    h.advance(0.1);

    let expected = ad * 0.2;
    assert!(
        (h.health(enemy) - (6000.0 - expected)).abs() < 1.0,
        "满层普攻应附带 20% AD={expected} 魔法伤害，实际剩余 {}",
        h.health(enemy)
    );
    h.finish();
}

/// 被动 6 秒未刷新则层数与攻速一并清除。
#[test]
fn irelia_passive_expires_after_duration() {
    let mut h = build_headless("irelia_passive_expire");
    let _ = h.add_enemy(Vec3::new(2000.0, 0.0, 0.0));

    h.cast_skill(0, Vec2::new(0.0, 0.0));
    h.advance(0.1);
    assert_eq!(fervor_charges(&h), Some(1));

    h.advance(6.5); // > 6s 持续时间
    assert_eq!(fervor_charges(&h), None, "6s 后热诚应过期清除");
    assert!(
        h.app.world().get::<BuffAttack>(h.champion).is_none(),
        "热诚过期后攻速加成应移除"
    );
    h.finish();
}
