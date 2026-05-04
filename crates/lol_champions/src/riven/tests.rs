#![cfg(test)]

use bevy::math::{Vec2, Vec3};

use crate::riven::Riven;
use crate::test_utils::*;

const EPSILON: f32 = 1e-3;

fn riven_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "riven",
        config_path: "characters/Riven/config.ron",
        skin_path: "characters/Riven/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::riven::PluginRiven);
        },
    }
}

fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Riven>(name, HarnessMode::Headless, &riven_config())
}

fn build_render(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Riven>(name, HarnessMode::Render, &riven_config())
}

#[test]
fn riven_q_cycles_through_three_real_stages() {
    let mut h = build_render("riven_q");

    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(3.5);

    let q_entity = h.skill_entity(0);

    assert_eq!(
        h.recast_window_stage(q_entity),
        Some(2),
        "第一次Q释放后应为第2阶段（总共3阶段，0=已用完）"
    );
    assert!(h.can_cast(0), "Q技能应该可以释放第二段");

    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(3.5);

    assert_eq!(
        h.recast_window_stage(q_entity),
        Some(3),
        "第二次Q释放后应为第3阶段（最后一次，可跃起击飞）"
    );
    assert!(h.can_cast(0), "Q技能应该可以释放第 3 段");

    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(1.0);

    assert!(
        !h.has_recast_window(q_entity),
        "Q技能三段全用完，RecastWindow应被移除"
    );
    assert!(!h.can_cast(0), "Q技能三段不能再释放");

    h.advance(6.0);
    h.finish();

    assert!(
        h.can_cast(0),
        "等待 3.5 + 3.5 + 1 + 6 = 14 秒后，13秒冷却应已结束"
    );
    assert!(
        h.position(h.champion).length() > 5.0,
        "三段Q位移后离原点应超过5单位"
    );
}

#[test]
fn riven_q_recast_window_expires_after_4_seconds() {
    let mut h = build_headless("riven_q_window");

    // Cast first Q
    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.4);

    let q_entity = h.skill_entity(0);

    assert_eq!(
        h.recast_window_stage(q_entity),
        Some(2),
        "第一次Q释放后应为第2阶段"
    );

    // Wait within 4s window: at t=3.9 (< 4s), can still recast Q2
    h.advance(3.5);
    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.1);

    assert_eq!(
        h.recast_window_stage(q_entity),
        Some(3),
        "3.9秒时释放Q2，应进入第3阶段"
    );

    // Q2 creates a NEW window with fresh 4s timer (expires at t=7.9)
    h.advance(0.15);
    assert!(
        h.has_recast_window(q_entity),
        "Q2创建了新窗口，新的4秒计时器未到期"
    );

    // Now wait for Q2's window to expire (need 7.9s from Q2 cast)
    h.advance(3.9); // total = 8.05s > 7.9s
    assert!(!h.has_recast_window(q_entity), "Q2的4秒窗口到期消失");
    h.finish();
}

#[test]
fn riven_w_hits_only_enemies_in_range() {
    let mut h = build_headless("riven_w");
    let enemy_near = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    let enemy_far = h.add_enemy(Vec3::new(420.0, 0.0, 0.0));
    let ally_near = h.add_ally(Vec3::new(60.0, 0.0, 0.0));
    let expected_damage = h
        .get_skill_value(
            1,
            "total_damage",
            1,
            |stat| {
                if stat == 2 { 100.0 } else { 0.0 }
            },
        )
        .expect("riven w damage should exist");
    let initial_near = h.health(enemy_near);
    let initial_far = h.health(enemy_far);
    let initial_ally = h.health(ally_near);

    h.cast_skill(1, Vec2::new(140.0, 0.0));

    h.advance(0.2);

    assert!(
        (initial_near - h.health(enemy_near) - expected_damage).abs() < EPSILON,
        "近距离敌人应受到W技能全额伤害（W范围260，距100在范围内）"
    );
    assert!(
        (h.health(enemy_far) - initial_far).abs() < EPSILON,
        "远处敌人（距420）应在W范围外不受伤害（W范围260）"
    );
    assert!(
        (h.health(ally_near) - initial_ally).abs() < EPSILON,
        "友军不应受W技能影响（W是敌方伤害技能）"
    );
    h.finish();
}

#[test]
fn riven_e_spawns_shield_and_dash_absorbs_damage() {
    let mut h = build_headless("riven_e");
    let enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));

    h.cast_skill(2, Vec2::new(140.0, 0.0)).advance(0.4);

    assert!(
        h.position(h.champion).length() > 2.0,
        "E技能释放后应有位移（冲刺约250单位）"
    );
    let initial_health = h.health(h.champion);
    let shield_val = h.shield_value().unwrap_or(0.0);
    assert!(
        shield_val > 80.0 && shield_val <= 100.0,
        "E技能护盾值应在80-100之间"
    );

    h.apply_damage(enemy, 60.0);

    assert!(
        (h.health(h.champion) - initial_health).abs() < EPSILON,
        "60点伤害应被护盾完全吸收，生命值不变"
    );
    let remaining_shield = h.shield_value().unwrap_or(0.0);
    assert!(
        remaining_shield > 20.0 && remaining_shield < shield_val,
        "护盾消耗后应剩余20以上"
    );

    h.apply_damage(enemy, 50.0);

    assert!(
        h.health(h.champion) < initial_health,
        "护盾耗尽后，生命值应下降"
    );
    h.finish();
}

#[test]
fn riven_r_starts_cooldown_without_moving_or_damaging() {
    let mut h = build_headless("riven_r");
    let _enemy = h.add_enemy(Vec3::new(100.0, 0.0, 0.0));
    let expected_mana_cost = h
        .spell(3)
        .expect("R spell missing")
        .spell_data
        .as_ref()
        .and_then(|s| s.mana.as_ref())
        .and_then(|m| m.first().copied())
        .unwrap_or(0.0);
    let initial_mana = h.mana();

    h.cast_skill(3, Vec2::new(140.0, 0.0)).advance(0.2);

    assert!(
        (h.mana() - (initial_mana - expected_mana_cost)).abs() < EPSILON,
        "R技能释放后法力值应减少"
    );
    assert!(!h.can_cast(3), "R技能应进入冷却（基础120秒）");
    assert!(
        h.position(h.champion).distance(Vec3::ZERO) < EPSILON,
        "R技能是Buff型技能，释放后位置应在原点不移动"
    );
    h.finish();
}
