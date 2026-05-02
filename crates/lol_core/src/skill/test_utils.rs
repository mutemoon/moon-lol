#![cfg(test)]

//! Shared test utilities for skill module integration tests.
//!
//! Provides common helpers for building headless Bevy test apps with the skill pipeline.

use std::collections::BTreeMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use league_utils::hash_key::HashKey;
use lol_base::prop::LoadHashKeyTrait;
use lol_base::spell::{DataSpell, Spell, ValuesEffect};
use lol_base::spell_calc::{CalculationPartEffectValue, CalculationSpell, CalculationType};

use crate::base::ability_resource::{AbilityResource, AbilityResourceType};
use crate::base::level::Level;
use crate::cooldown::PluginCooldown;
use crate::damage::PluginDamage;
use crate::life::{Health, PluginLife};
use crate::movement::PluginMovement;
use crate::team::Team;

/// Default test frame rate
pub const TEST_FPS: f32 = 30.0;

/// Create a mana resource with the given current value.
pub fn make_mana(value: f32) -> AbilityResource {
    AbilityResource {
        ar_type: AbilityResourceType::Mana,
        value,
        max: 100.0,
        base: 0.0,
        per_level: 0.0,
        base_static_regen: 0.0,
        regen_per_level: 0.0,
    }
}

/// Construct a spell handle from a numeric key.
pub fn spell_handle(key: u32) -> Handle<Spell> {
    Handle::from(HashKey::<Spell>::from(key))
}

/// Build a headless Bevy App with the core skill pipeline plugins.
///
/// Includes: MinimalPlugins, AssetPlugin, PluginAction, PluginCooldown,
/// PluginDamage, PluginLife, PluginMovement, PluginSkill.
/// Configured with `TimeUpdateStrategy::ManualDuration` for deterministic testing.
pub fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(crate::action::PluginAction);
    app.add_plugins(PluginCooldown);
    app.add_plugins(PluginDamage);
    app.add_plugins(PluginLife);
    app.add_plugins(PluginMovement);
    app.add_plugins(crate::skill::PluginSkill);
    app.init_asset::<Spell>();
    app.insert_resource(Time::<Fixed>::from_hz(TEST_FPS as f64));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / TEST_FPS as f64,
    )));
    app
}

/// Spawn a caster entity with minimal skill-related components.
pub fn spawn_caster(world: &mut World, team: Team, position: Vec3) -> Entity {
    world
        .spawn((
            team,
            Transform::from_translation(position),
            Level::default(),
            crate::skill::SkillPoints(1),
            crate::skill::Skills::default(),
            make_mana(100.0),
        ))
        .id()
}

/// Spawn a target entity (enemy with health).
pub fn spawn_target(world: &mut World, team: Team, position: Vec3, hp: f32) -> Entity {
    world
        .spawn((team, Transform::from_translation(position), Health::new(hp)))
        .id()
}

/// Advance the app by `seconds` of simulated time using FixedUpdate schedule.
pub fn advance_time(app: &mut App, seconds: f32) {
    let ticks = (seconds * TEST_FPS).ceil() as u32;
    for _ in 0..ticks {
        let mut time = app.world_mut().resource_mut::<Time<Fixed>>();
        time.advance_by(Duration::from_secs_f64(1.0 / TEST_FPS as f64));
        drop(time);
        app.world_mut().run_schedule(FixedUpdate);
    }
}

/// Register a minimal spell with damage and mana cost into the app's asset store.
pub fn register_test_spell(
    app: &mut App,
    spell_key: u32,
    damage_key: u32,
    mana_cost: f32,
    damage_values: Vec<f32>,
) {
    use lol_base::spell_calc::CalculationPart;

    let mut calculations = BTreeMap::new();
    calculations.insert(
        damage_key,
        CalculationType::CalculationSpell(CalculationSpell {
            formula_parts: Some(vec![CalculationPart::CalculationPartEffectValue(
                CalculationPartEffectValue {
                    effect_index: Some(1),
                },
            )]),
            multiplier: None,
            precision: None,
        }),
    );

    let effect_values = ValuesEffect {
        values: Some(damage_values),
    };

    app.world_mut().resource_mut::<Assets<Spell>>().add_hash(
        spell_key,
        Spell {
            spell_data: Some(DataSpell {
                calculations: Some(calculations),
                effect_amounts: Some(vec![effect_values]),
                data_values: None,
                mana: Some(vec![mana_cost; 6]),
                missile_spec: None,
                hit_bone_name: None,
                missile_speed: None,
                missile_effect_key: None,
                cast_type: None,
            }),
        },
    );
}
