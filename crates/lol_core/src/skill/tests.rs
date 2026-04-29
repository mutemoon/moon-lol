#![cfg(test)]

use std::collections::BTreeMap;

use league_utils::hash_bin;
use lol_base::spell::{DataSpell, Spell, ValuesData, ValuesEffect};
use lol_base::spell_calc::{
    CalculationPart, CalculationPartEffectValue, CalculationPartNamedDataValue,
    CalculationPartStatCoefficient, CalculationSpell, CalculationType,
};

use super::*;

fn create_mock_spell(
    calculations: BTreeMap<u32, CalculationType>,
    effect_amounts: Option<Vec<ValuesEffect>>,
    data_values: Option<Vec<ValuesData>>,
) -> Spell {
    Spell {
        spell_data: Some(DataSpell {
            calculations: Some(calculations),
            effect_amounts,
            data_values,
            mana: None,
            missile_spec: None,
            hit_bone_name: None,
            missile_speed: None,
            missile_effect_key: None,
            cast_type: None,
        }),
    }
}

#[test]
fn test_effect_value_calculation() {
    // Setup
    let hash = 123;
    let effect_index = 1;
    let expected_value_lvl1 = 10.0;
    let expected_value_lvl2 = 20.0;

    let calc_part = CalculationPart::CalculationPartEffectValue(CalculationPartEffectValue {
        effect_index: Some(effect_index),
    });

    let calc = CalculationType::CalculationSpell(CalculationSpell {
        formula_parts: Some(vec![calc_part]),
        multiplier: None,
        precision: None,
    });

    let mut calculations = BTreeMap::new();
    calculations.insert(hash, calc);

    let effect_amounts = vec![ValuesEffect {
        values: Some(vec![expected_value_lvl1, expected_value_lvl2, 30.0]),
    }];

    let spell = create_mock_spell(calculations, Some(effect_amounts), None);

    // Test Level 1
    let result = get_skill_value(&spell, hash, 1, |_| 0.0);
    assert_eq!(result, Some(expected_value_lvl1));

    // Test Level 2
    let result = get_skill_value(&spell, hash, 2, |_| 0.0);
    assert_eq!(result, Some(expected_value_lvl2));
}

#[test]
fn test_stat_by_coefficient_calculation() {
    // Setup
    let hash = 456;
    let stat_id = 2; // e.g., Attack Damage
    let coefficient = 1.5;
    let stat_value = 100.0;
    let expected_value = stat_value * coefficient;

    let calc_part =
        CalculationPart::CalculationPartStatCoefficient(CalculationPartStatCoefficient {
            stat: Some(stat_id),
            coefficient: Some(coefficient),
            stat_formula: None,
        });

    let calc = CalculationType::CalculationSpell(CalculationSpell {
        formula_parts: Some(vec![calc_part]),
        multiplier: None,
        precision: None,
    });

    let mut calculations = BTreeMap::new();
    calculations.insert(hash, calc);

    let spell = create_mock_spell(calculations, None, None);

    // Test
    let result = get_skill_value(&spell, hash, 1, |id| {
        if id == stat_id { stat_value } else { 0.0 }
    });
    assert_eq!(result, Some(expected_value));
}

#[test]
fn test_named_data_value_calculation() {
    // Setup
    let hash = 789;
    let data_name = "BaseDamage";
    let data_name_hash = hash_bin(data_name);
    let expected_value = 50.0;

    let calc_part = CalculationPart::CalculationPartNamedDataValue(CalculationPartNamedDataValue {
        data_value: data_name_hash,
    });

    let calc = CalculationType::CalculationSpell(CalculationSpell {
        formula_parts: Some(vec![calc_part]),
        multiplier: None,
        precision: None,
    });

    let mut calculations = BTreeMap::new();
    calculations.insert(hash, calc);

    let data_values = vec![ValuesData {
        name: data_name.to_string(),
        values: Some(vec![expected_value, 60.0, 70.0]),
    }];

    let spell = create_mock_spell(calculations, None, Some(data_values));

    // Test
    let result = get_skill_value(&spell, hash, 1, |_| 0.0);
    assert_eq!(result, Some(expected_value));
}
