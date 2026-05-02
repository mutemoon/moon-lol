#![cfg(test)]
//! Logic and render/video tests for Fiora skills.
//!
//! Headless mode uses mock spells; render mode uses real `ConfigCharacterRecord` + skin.

use std::collections::BTreeMap;

use bevy::math::Vec2;
use lol_base::spell::{DataSpell, Spell, ValuesData};
use lol_base::spell_calc::{
    CalculationPart, CalculationPartNamedDataValue, CalculationSpell, CalculationType,
};
use lol_core::skill::SkillCooldownMode;

use crate::fiora::Fiora;
use crate::test_utils::*;

const FIORA_Q_KEY: &str = "Characters/Fiora/Spells/FioraQ/FioraQ";
const FIORA_W_KEY: &str = "Characters/Fiora/Spells/FioraW/FioraW";
const FIORA_E_KEY: &str = "Characters/Fiora/Spells/FioraE/FioraE";
const FIORA_R_KEY: &str = "Characters/Fiora/Spells/FioraR/FioraR";
const FIORA_PASSIVE_KEY: &str = "Characters/Fiora/Spells/FioraPassive/FioraPassive";

fn fiora_spell_keys() -> SpellKeySet {
    SpellKeySet {
        q: FIORA_Q_KEY,
        w: FIORA_W_KEY,
        e: FIORA_E_KEY,
        r: FIORA_R_KEY,
        passive: FIORA_PASSIVE_KEY,
    }
}

fn fiora_mock_spell() -> Spell {
    let mut calculations = BTreeMap::new();
    calculations.insert(
        "TotalDamage".to_string(),
        CalculationType::CalculationSpell(CalculationSpell {
            formula_parts: Some(vec![CalculationPart::CalculationPartNamedDataValue(
                CalculationPartNamedDataValue {
                    data_value: "TotalDamage".to_string(),
                },
            )]),
            multiplier: None,
            precision: None,
        }),
    );
    Spell {
        spell_data: Some(DataSpell {
            calculations: Some(calculations),
            data_values: Some(vec![ValuesData {
                name: "TotalDamage".into(),
                values: Some(vec![130.0; 6]),
            }]),
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

fn fiora_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "fiora",
        config_path: "characters/fiora/config.ron",
        skin_path: "characters/fiora/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::fiora::PluginFiora);
        },
        make_mock_spell: fiora_mock_spell,
        cooldown_mode_for: |_| SkillCooldownMode::Manual,
        spell_keys: fiora_spell_keys(),
    }
}

fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Fiora>(name, HarnessMode::Headless, &fiora_config())
}
fn build_render(name: &str, max_frames: u32) -> ChampionTestHarness {
    ChampionTestHarness::build::<Fiora>(name, HarnessMode::Render { max_frames }, &fiora_config())
}

#[test]
fn fiora_harness_builds_and_settles() {
    let _h = build_headless("fiora_smoke");
}

#[test]
fn fiora_q_writes_video() {
    run_render_test(
        || build_render("fiora_q_writes_video", 120),
        |h| {
            h.cast_skill(0, Vec2::new(170.0, 0.0)).advance(0.4);
        },
    );
}
#[test]
fn fiora_w_writes_video() {
    run_render_test(
        || build_render("fiora_w_writes_video", 120),
        |h| {
            h.cast_skill(1, Vec2::new(170.0, 0.0)).advance(0.4);
        },
    );
}
#[test]
fn fiora_e_writes_video() {
    run_render_test(
        || build_render("fiora_e_writes_video", 120),
        |h| {
            h.cast_skill(2, Vec2::new(170.0, 0.0)).advance(0.4);
        },
    );
}
#[test]
fn fiora_r_writes_video() {
    run_render_test(
        || build_render("fiora_r_writes_video", 140),
        |h| {
            h.cast_skill(3, Vec2::new(170.0, 0.0)).advance(0.4);
        },
    );
}
