use lol_base::spell::Spell;
use lol_base::spell_calc::{
    CalculationPart, CalculationPartEffectValue, CalculationPartNamedDataValue,
    CalculationPartStatCoefficient, CalculationPartStatNamedDataValue, CalculationPartStatSub,
    CalculationType,
};

pub fn get_skill_value(
    skill_object: &Spell,
    name: &str,
    level: usize,
    get_stat: impl Fn(u8) -> f32,
) -> Option<f32> {
    let spell = skill_object.spell_data.as_ref()?;
    let calculations = spell.calculations.as_ref()?;
    let calculation = calculations.get(name)?;

    let CalculationType::CalculationSpell(calc) = calculation;
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

fn get_named_data_value(skill_object: &Spell, target_name: &str, level: usize) -> Option<f32> {
    let spell_data = skill_object.spell_data.as_ref()?;
    let data_values = spell_data.data_values.as_ref()?;
    let norm_target = target_name.to_lowercase().replace('_', "");

    for dv in data_values {
        let norm_name = dv.name.to_lowercase().replace('_', "");
        if norm_name != norm_target {
            continue;
        }
        let values = dv.values.as_ref()?;
        let lvl_idx = if level > 0 { level - 1 } else { 0 };
        return Some(*values.get(lvl_idx).unwrap_or(&0.0));
    }
    None
}

fn get_effect_value(skill_object: &Spell, effect_index: Option<i32>, level: usize) -> Option<f32> {
    let index = effect_index.unwrap_or(1) - 1;
    let spell_data = skill_object.spell_data.as_ref()?;
    let effect_amounts = spell_data.effect_amounts.as_ref()?;
    let effect_amount = effect_amounts.get(index as usize)?;
    let values = effect_amount.values.as_ref()?;
    let lvl_idx = if level > 0 { level - 1 } else { 0 };
    Some(*values.get(lvl_idx).unwrap_or(&0.0))
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
        }) => get_effect_value(skill_object, *effect_index, level).unwrap_or(0.0),
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
        }) => get_named_data_value(skill_object, data_value, level).unwrap_or(0.0),
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
            let val = get_named_data_value(skill_object, data_value, level).unwrap_or(0.0);
            get_stat(stat) * val
        }
    }
}
