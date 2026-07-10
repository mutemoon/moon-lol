#![cfg(test)]

//! Fiora Q (Lunge) 单元测试。
//!
//! Q 的语义：向指针方向位移，位移停止后戳刺最近的单位；
//! 有敌方英雄时优先戳英雄，而不是像 Riven Q 那样对路径上的敌人造成碰撞伤害。

use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Entity, Transform};
use lol_core::action::{Action, CommandAction};
use lol_core::attack::{AttackState, AttackStatus};
use lol_core::attack_auto::AttackAuto;
use lol_core::base::bounding::Bounding;
use lol_core::base::direction::Direction;
use lol_core::life::Health;
use lol_core::skill::CoolDown;
use lol_core::team::Team;

use super::tests::build_headless;
use crate::fiora::passive::{FIORA_PASSIVE_ACTIVE_DURATION, FIORA_PASSIVE_DURATION, Vital};
use crate::test_utils::ChampionTestHarness;

const EPSILON: f32 = 1e-3;

/// 生成一个非英雄的敌方单位（小兵），用于测试“优先戳英雄”的目标筛选。
///
/// 测试 harness 的 `add_enemy` 只能生成英雄，这里手动生成一个不带 `Champion`
/// 组件的敌方实体来模拟小兵。
fn add_minion(h: &mut ChampionTestHarness, position: Vec3) -> Entity {
    h.app
        .world_mut()
        .spawn((
            Team::Chaos,
            Transform::from_translation(position),
            Health::new(6000.0),
            lol_core::damage::Armor(0.0),
        ))
        .id()
}

/// Q 应在位移停止后戳刺，而不是施法瞬间就造成伤害。
#[test]
fn fiora_q_strikes_after_dash_not_at_cast() {
    let mut h = build_headless("fiora_q_timing");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let hp_before = h.health(enemy);

    // 位移 300 单位 @ 1000 速度 ≈ 0.3s；0.1s 时位移尚未结束，不应有戳刺伤害
    h.cast_skill(0, Vec2::new(300.0, 0.0)).advance(0.1);
    assert!(
        (h.health(enemy) - hp_before).abs() < EPSILON,
        "位移结束前不应造成伤害（戳刺发生在位移停止后），实际血量: {}",
        h.health(enemy)
    );

    // 位移结束后应戳刺最近单位
    h.advance(0.5);
    assert!(
        h.health(enemy) < hp_before,
        "位移停止后应戳刺最近敌人造成伤害"
    );

    h.finish();
}

/// 有敌方英雄时优先戳英雄，且不对位移路径上的敌人造成碰撞伤害（区别于 Riven Q）。
#[test]
fn fiora_q_prioritizes_champion_over_closer_minion() {
    let mut h = build_headless("fiora_q_priority");
    // 小兵在位移路径上，且更靠近位移终点 (300,0,0)
    let minion = add_minion(&mut h, Vec3::new(200.0, 0.0, 0.0));
    // 英雄在小兵侧面，距位移终点更远（约 180），但是英雄
    let champion = h.add_enemy(Vec3::new(200.0, 0.0, 150.0));

    let minion_hp_before = h.health(minion);
    let champ_hp_before = h.health(champion);

    h.cast_skill(0, Vec2::new(300.0, 0.0)).advance(0.1);
    // 位移路径上不应有碰撞伤害（区别于 Riven Q 的路径碰撞伤害）
    assert!(
        (h.health(minion) - minion_hp_before).abs() < EPSILON,
        "位移期间不应像 Riven 那样对路径上的敌人造成碰撞伤害，小兵血量: {}",
        h.health(minion)
    );

    h.advance(0.5);
    // 有英雄时优先戳英雄
    assert!(
        h.health(champion) < champ_hp_before,
        "有敌方英雄时应优先戳英雄"
    );
    assert!(
        (h.health(minion) - minion_hp_before).abs() < EPSILON,
        "优先戳英雄时不应误伤更近的小兵，小兵血量: {}",
        h.health(minion)
    );

    h.finish();
}

/// 没有敌方英雄时，戳刺最近的单位（小兵）。
#[test]
fn fiora_q_strikes_nearest_unit_when_no_champion() {
    let mut h = build_headless("fiora_q_fallback");
    let minion = add_minion(&mut h, Vec3::new(200.0, 0.0, 0.0));
    let hp_before = h.health(minion);

    h.cast_skill(0, Vec2::new(300.0, 0.0)).advance(0.1);
    assert!(
        (h.health(minion) - hp_before).abs() < EPSILON,
        "位移结束前不应造成伤害"
    );

    h.advance(0.5);
    assert!(
        h.health(minion) < hp_before,
        "没有敌方英雄时，应戳刺最近的单位（小兵）"
    );

    h.finish();
}

/// 戳刺命中判定应包含敌人的碰撞半径：以敌人边缘是否进入戳刺范围为准，
/// 而不是仅看敌人中心点。
#[test]
fn fiora_q_strike_includes_target_bounding_radius() {
    let mut h = build_headless("fiora_q_bounding");
    // 位移终点为 (300,0,0)。敌人中心距终点 220，刚好超出戳刺半径 200
    // （仅按中心点判定会 miss）；但敌人碰撞半径 35，边缘距离 220-35=185 在范围内，应命中。
    let enemy = h.add_enemy(Vec3::new(520.0, 0.0, 0.0));
    h.app.world_mut().entity_mut(enemy).insert(Bounding {
        radius: 35.0,
        height: 200.0,
    });

    let hp_before = h.health(enemy);

    h.cast_skill(0, Vec2::new(300.0, 0.0)).advance(0.1);
    assert!(
        (h.health(enemy) - hp_before).abs() < EPSILON,
        "位移结束前不应造成伤害"
    );

    h.advance(0.5);
    assert!(
        h.health(enemy) < hp_before,
        "戳刺命中判定应包含敌人碰撞半径（边缘进入范围即命中），敌人血量: {}",
        h.health(enemy)
    );

    h.finish();
}

/// Q 施法应重置普攻（`CommandAttackReset` 会移除当前 `AttackState` 并以保存的目标
/// 立即重新起手一段全新 Windup，其 `end_time` 应被刷新到更晚）。
#[test]
fn fiora_q_resets_attack_timer() {
    let mut h = build_headless("fiora_q_attack_reset");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0)); // 在普攻射程 150 内
    // 自动攻击要求目标有碰撞体
    h.app.world_mut().entity_mut(enemy).insert(Bounding {
        radius: 35.0,
        height: 200.0,
    });

    // 起手一次普攻，进入 windup(0.2s)
    h.app.world_mut().trigger(CommandAction {
        entity: h.champion,
        action: Action::Attack(enemy),
    });
    h.advance(0.1);
    let end_before = match h.app.world().get::<AttackState>(h.champion) {
        Some(s) => match s.status {
            AttackStatus::Windup { end_time, .. } => end_time,
            _ => panic!("普攻起手后应处于 Windup"),
        },
        None => panic!("普攻起手后应有 AttackState"),
    };

    // 移除自动攻击组件，避免 update_attack_auto 干扰，隔离 Q 的重置效果。
    h.app
        .world_mut()
        .entity_mut(h.champion)
        .remove::<AttackAuto>();

    // 施放 Q：应触发 CommandAttackReset，重新起手一段全新 Windup（end_time 更晚）。
    h.cast_skill(0, Vec2::new(300.0, 0.0)).advance(0.05);
    let end_after = match h.app.world().get::<AttackState>(h.champion) {
        Some(s) => match s.status {
            AttackStatus::Windup { end_time, .. } => end_time,
            _ => panic!("Q 重置后应重新处于 Windup"),
        },
        None => panic!("Q 重置后应有全新 AttackState"),
    };

    assert!(
        end_after > end_before,
        "Q 应重置普攻（Windup end_time 刷新到更晚）：{} > {}",
        end_after,
        end_before
    );

    h.finish();
}

/// Q 命中敌人时应退还没收冷却（`CDRefundPercent`，ron 中为 0.5）。
#[test]
fn fiora_q_hit_refunds_cooldown() {
    // 命中：位移终点附近有敌人 -> 戳刺命中 -> 退还没收
    let mut h_hit = build_headless("fiora_q_cd_refund_hit");
    let _enemy = h_hit.add_enemy(Vec3::new(450.0, 0.0, 0.0));
    h_hit.cast_skill(0, Vec2::new(300.0, 0.0)).advance(0.6);
    let remaining_hit = h_hit
        .app
        .world()
        .get::<CoolDown>(h_hit.skill_entity(0))
        .expect("Q 技能实体应有 CoolDown")
        .timer
        .as_ref()
        .expect("Q 命中后冷却应已启动")
        .remaining_secs();

    // 未命中：无敌人 -> 不戳刺 -> 不退款
    let mut h_miss = build_headless("fiora_q_cd_refund_miss");
    h_miss.cast_skill(0, Vec2::new(300.0, 0.0)).advance(0.6);
    let remaining_miss = h_miss
        .app
        .world()
        .get::<CoolDown>(h_miss.skill_entity(0))
        .expect("Q 技能实体应有 CoolDown")
        .timer
        .as_ref()
        .expect("Q 施法后冷却应已启动")
        .remaining_secs();

    assert!(
        remaining_hit < remaining_miss * 0.6,
        "命中后 Q 冷却应大幅退还没收：命中剩余 {:.2}s，未命中剩余 {:.2}s",
        remaining_hit,
        remaining_miss
    );

    h_hit.finish();
    h_miss.finish();
}

/// Q 戳刺命中「方向匹配的活跃要害」时，物理伤害应翻倍。
///
/// 用两个 harness 对比：
/// - 非匹配方向（Up）：被动不触发、Q 不翻倍 -> 基础物理伤害 X。
/// - 匹配方向（Left，菲奥娜在敌人西侧戳刺）：Q 翻倍 2X + 被动真伤(5%maxHP=300)。
/// 断言「匹配 - 非匹配 > 350」，即超出被动真伤部分即 Q 翻倍的额外物理伤害。
#[test]
fn fiora_q_damage_doubled_on_vital() {
    // 菲奥娜从原点位移到 (300,0)，戳刺 (450,0) 的敌人：
    // source=(300,0) 在 target=(450,0) 西侧 -> is_in_direction 命中 Left。
    let run = |direction: Direction, label: &str| -> f32 {
        let mut h = build_headless(label);
        let enemy = h.add_enemy(Vec3::new(450.0, 0.0, 0.0));
        h.app.world_mut().entity_mut(enemy).insert(Bounding {
            radius: 35.0,
            height: 200.0,
        });
        // 手动挂指定方向的 Vital（绕过被动随机方向），并等其进入活跃态
        h.app.world_mut().entity_mut(enemy).insert(Vital::new(
            direction,
            FIORA_PASSIVE_ACTIVE_DURATION,
            FIORA_PASSIVE_DURATION,
        ));
        h.advance(1.8);

        let hp_before = h.health(enemy);
        h.cast_skill(0, Vec2::new(300.0, 0.0)).advance(0.6);
        let damage = hp_before - h.health(enemy);
        h.finish();
        damage
    };

    let d_match = run(Direction::Left, "fiora_q_vital_match");
    let d_base = run(Direction::Up, "fiora_q_vital_nomatch");

    assert!(
        d_match - d_base > 350.0,
        "命中匹配要害时应造成翻倍物理伤害（超出被动真伤 300 的部分）：匹配 {:.1}，非匹配 {:.1}，差 {:.1}",
        d_match,
        d_base,
        d_match - d_base
    );
}
