#![cfg(test)]
//! Logic and render/video tests for Riven skills.
//!
//! Headless mode uses mock spells; render mode uses real `ConfigCharacterRecord` + skin.

use std::collections::BTreeMap;

use bevy::math::{Vec2, Vec3};
use league_utils::hash_bin;
use lol_base::spell::{DataSpell, Spell, ValuesData};
use lol_base::spell_calc::{
    CalculationPart, CalculationPartNamedDataValue, CalculationSpell, CalculationType,
};
use lol_core::skill::{SkillCooldownMode, SkillRecastWindow, SkillSlot, Skills, get_skill_value};

use crate::riven::Riven;
use crate::test_utils::*;

const RIVEN_Q_KEY: &str = "Characters/Riven/Spells/RivenTriCleaveAbility/RivenTriCleave";
const RIVEN_W_KEY: &str = "Characters/Riven/Spells/RivenMartyrAbility/RivenMartyr";
const RIVEN_E_KEY: &str = "Characters/Riven/Spells/RivenFeintAbility/RivenFeint";
const RIVEN_R_KEY: &str = "Characters/Riven/Spells/RivenFengShuiEngineAbility/RivenFengShuiEngine";

const EPSILON: f32 = 1e-3;

fn riven_spell_keys() -> SpellKeySet {
    SpellKeySet {
        q: RIVEN_Q_KEY,
        w: RIVEN_W_KEY,
        e: RIVEN_E_KEY,
        r: RIVEN_R_KEY,
        passive: "Characters/Riven/Spells/RivenPassiveAbility/RivenPassive",
    }
}

fn riven_mock_spell() -> Spell {
    let mut calculations = BTreeMap::new();
    let make_calc = |name: &str| {
        CalculationType::CalculationSpell(CalculationSpell {
            formula_parts: Some(vec![CalculationPart::CalculationPartNamedDataValue(
                CalculationPartNamedDataValue {
                    data_value: name.to_string(),
                },
            )]),
            multiplier: None,
            precision: None,
        })
    };
    calculations.insert("m_damage".to_string(), make_calc("m_damage"));
    calculations.insert("total_damage".to_string(), make_calc("total_damage"));
    calculations.insert(
        "first_slash_damage".to_string(),
        make_calc("first_slash_damage"),
    );
    calculations.insert("shield_strength".to_string(), make_calc("shield_strength"));
    Spell {
        spell_data: Some(DataSpell {
            calculations: Some(calculations),
            data_values: Some(vec![
                ValuesData {
                    name: "mDamage".into(),
                    values: Some(vec![130.0; 6]),
                },
                ValuesData {
                    name: "TotalDamage".into(),
                    values: Some(vec![130.0; 6]),
                },
                ValuesData {
                    name: "FirstSlashDamage".into(),
                    values: Some(vec![130.0; 6]),
                },
                ValuesData {
                    name: "ShieldStrength".into(),
                    values: Some(vec![100.0; 6]),
                },
            ]),
            effect_amounts: None,
            mana: None,
            missile_spec: None,
            hit_bone_name: None,
            missile_speed: None,
            missile_effect_key: None,
            cast_type: None,
        }),
    }
}

fn riven_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "riven",
        config_path: "characters/riven/config.ron",
        skin_path: "characters/riven/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::riven::PluginRiven);
        },
        make_mock_spell: riven_mock_spell,
        cooldown_mode_for: |slot| {
            if slot == SkillSlot::Q {
                SkillCooldownMode::Manual
            } else {
                SkillCooldownMode::AfterCast
            }
        },
        spell_keys: riven_spell_keys(),
    }
}

fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Riven>(name, HarnessMode::Headless, &riven_config())
}

fn build_render(name: &str, max_frames: u32) -> ChampionTestHarness {
    ChampionTestHarness::build::<Riven>(name, HarnessMode::Render { max_frames }, &riven_config())
}

fn w_damage_key() -> &'static str {
    "total_damage"
}

// ═══════════════════════════════════════════════════════════
// Headless tests
// ═══════════════════════════════════════════════════════════

#[test]
fn riven_q_cycles_through_three_real_stages() {
    let mut h = build_headless("riven_q");
    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.4);
    let q_entity = (h
        .app
        .world()
        .get::<Skills>(h.champion)
        .expect("Skills missing"))[0];
    let q_stage = h
        .app
        .world()
        .get::<SkillRecastWindow>(q_entity)
        .map(|w| w.stage);
    if q_stage != Some(2) {
        h.print_skill_logs();
    }
    assert_eq!(q_stage, Some(2));
    assert!(h.cooldown_finished(0));
    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.4);
    assert_eq!(
        h.app
            .world()
            .get::<SkillRecastWindow>(q_entity)
            .map(|w| w.stage),
        Some(3)
    );
    assert!(h.cooldown_finished(0));
    h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.1);
    assert!(h.app.world().get::<SkillRecastWindow>(q_entity).is_none());
    assert!(!h.cooldown_finished(0));
    h.advance(10.1);
    assert!(h.cooldown_finished(0));
    assert!(h.position(h.champion).length() > 5.0);
}

#[test]
fn riven_w_hits_only_enemies_in_range() {
    let mut h = build_headless("riven_w");
    let expected_damage = get_skill_value(
        &h.spell(1).expect("W spell missing"),
        w_damage_key(),
        1,
        |stat| if stat == 2 { 100.0 } else { 0.0 },
    )
    .expect("riven w damage should exist");
    let initial_near = h.health(h.enemy_near);
    let initial_far = h.health(h.enemy_far);
    let initial_ally = h.health(h.ally_near);
    h.cast_skill(1, Vec2::new(140.0, 0.0));
    h.advance(0.2);
    assert!((initial_near - h.health(h.enemy_near) - expected_damage).abs() < EPSILON);
    assert!((h.health(h.enemy_far) - initial_far).abs() < EPSILON);
    assert!((h.health(h.ally_near) - initial_ally).abs() < EPSILON);
}

#[test]
fn riven_e_spawns_shield_and_dash_absorbs_damage() {
    let mut h = build_headless("riven_e");
    h.cast_skill(2, Vec2::new(140.0, 0.0)).advance(0.4);
    assert!(h.position(h.champion).length() > 2.0);
    let initial_health = h.health(h.champion);
    let shield_val = h.shield_value().unwrap_or(0.0);
    assert!(shield_val > 80.0 && shield_val <= 100.0);
    h.apply_damage(60.0);
    assert!((h.health(h.champion) - initial_health).abs() < EPSILON);
    let remaining_shield = h.shield_value().unwrap_or(0.0);
    assert!(remaining_shield > 20.0 && remaining_shield < shield_val);
    h.apply_damage(50.0);
    assert!(h.health(h.champion) < initial_health);
}

#[test]
fn riven_r_starts_cooldown_without_moving_or_damaging() {
    let mut h = build_headless("riven_r");
    let expected_mana_cost = h
        .spell(3)
        .expect("R spell missing")
        .spell_data
        .as_ref()
        .and_then(|s| s.mana.as_ref())
        .and_then(|m| m.first().copied())
        .unwrap_or(0.0);
    let initial_mana = h.mana();
    let initial_enemy_hp = h.health(h.enemy_near);
    h.cast_skill(3, Vec2::new(140.0, 0.0)).advance(0.2);
    assert!((h.mana() - (initial_mana - expected_mana_cost)).abs() < EPSILON);
    assert!(!h.cooldown_finished(3));
    assert!(h.position(h.champion).distance(Vec3::ZERO) < EPSILON);
    assert!((h.health(h.enemy_near) - initial_enemy_hp).abs() < EPSILON);
}

// ═══════════════════════════════════════════════════════════
// Render tests
// ═══════════════════════════════════════════════════════════

#[test]
fn riven_q_writes_video() {
    run_render_test(
        || build_render("riven_q_writes_video", 180),
        |h| {
            h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.4);
            h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.4);
            h.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.1);
        },
    );
}
#[test]
fn riven_w_writes_video() {
    run_render_test(
        || build_render("riven_w_writes_video", 120),
        |h| {
            h.cast_skill(1, Vec2::new(140.0, 0.0)).advance(0.2);
        },
    );
}
#[test]
fn riven_e_writes_video() {
    run_render_test(
        || build_render("riven_e_writes_video", 120),
        |h| {
            h.cast_skill(2, Vec2::new(140.0, 0.0)).advance(0.4);
            h.apply_damage(60.0);
            h.apply_damage(50.0);
        },
    );
}
#[test]
fn riven_r_writes_video() {
    run_render_test(
        || build_render("riven_r_writes_video", 140),
        |h| {
            h.cast_skill(3, Vec2::new(140.0, 0.0)).advance(0.2);
        },
    );
}
