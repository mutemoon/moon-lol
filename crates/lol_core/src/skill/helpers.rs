use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_base::spell_calc::{
    CalculationPart, CalculationPartEffectValue, CalculationPartNamedDataValue,
    CalculationPartStatCoefficient, CalculationPartStatNamedDataValue, CalculationPartStatSub,
    CalculationType,
};

pub fn get_skill_value(
    skill_object: &Spell,
    hash: u32,
    level: usize,
    get_stat: impl Fn(u8) -> f32,
) -> Option<f32> {
    let spell = skill_object.spell_data.as_ref()?;
    let calculations = spell.calculations.as_ref()?;
    let calculation = calculations.get(&hash)?;

    match calculation {
        CalculationType::CalculationSpell(calc) => {
            let mut value = 0.0;
            if let Some(parts) = &calc.formula_parts {
                for part in parts {
                    value += calculate_part(part, skill_object, level, &get_stat);
                }
            }

            if let Some(multiplier) = &calc.multiplier {
                value *= calculate_part(multiplier, skill_object, level, &get_stat);
            }
            Some(value)
        }
    }
}

fn calculate_part(
    part: &CalculationPart,
    skill_object: &Spell,
    level: usize,
    get_stat: &impl Fn(u8) -> f32,
) -> f32 {
    match part {
        CalculationPart::CalculationPartEffectValue(CalculationPartEffectValue {
            effect_index,
        }) => {
            let index = effect_index.unwrap_or(1) - 1;
            if let Some(effect_amount) = skill_object
                .spell_data
                .as_ref()
                .and_then(|s| s.effect_amounts.as_ref())
                .and_then(|v| v.get(index as usize))
            {
                if let Some(values) = &effect_amount.values {
                    // level is 1-based, so index is level - 1
                    // Ensure level is at least 1
                    let lvl_idx = if level > 0 { level - 1 } else { 0 };
                    return *values.get(lvl_idx).unwrap_or(&0.0);
                }
            }
            0.0
        }
        CalculationPart::CalculationPartStatCoefficient(CalculationPartStatCoefficient {
            stat,
            coefficient,
            ..
        }) => {
            let stat = stat.unwrap_or(0);
            let coefficient = coefficient.unwrap_or(0.0);
            get_stat(stat) * coefficient
        }
        CalculationPart::CalculationPartNamedDataValue(CalculationPartNamedDataValue {
            data_value,
        }) => {
            if let Some(data_values) = skill_object
                .spell_data
                .as_ref()
                .and_then(|s| s.data_values.as_ref())
            {
                for dv in data_values {
                    // Check if hash of name matches data_value
                    let hash = hash_bin(&dv.name);
                    if hash == *data_value {
                        if let Some(values) = &dv.values {
                            let lvl_idx = if level > 0 { level - 1 } else { 0 };
                            return *values.get(lvl_idx).unwrap_or(&0.0);
                        }
                    }
                }
            }
            0.0
        }
        CalculationPart::CalculationPartStatSub(CalculationPartStatSub {
            stat, subpart, ..
        }) => {
            let stat_val = stat.unwrap_or(0);
            let sub_val = if let Some(sub) = subpart {
                calculate_part(sub, skill_object, level, get_stat)
            } else {
                0.0
            };
            get_stat(stat_val) * sub_val
        }
        CalculationPart::CalculationPartStatNamedDataValue(CalculationPartStatNamedDataValue {
            stat,
            data_value,
            ..
        }) => {
            let stat = stat.unwrap_or(0);
            let mut data_val = 0.0;
            if let Some(data_values) = skill_object
                .spell_data
                .as_ref()
                .and_then(|s| s.data_values.as_ref())
            {
                for dv in data_values {
                    let hash = hash_bin(&dv.name);
                    if hash == *data_value {
                        if let Some(values) = &dv.values {
                            let lvl_idx = if level > 0 { level - 1 } else { 0 };
                            data_val = *values.get(lvl_idx).unwrap_or(&0.0);
                            break;
                        }
                    }
                }
            }
            get_stat(stat) * data_val
        }
    }
}
