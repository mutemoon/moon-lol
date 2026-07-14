use lol_base::spell::Spell;
use lol_base::spell_calc::{
    CalculationPart, CalculationPartEffectValue, CalculationPartNamedDataValue,
    CalculationPartStatCoefficient, CalculationPartStatNamedDataValue, CalculationPartStatSub,
    CalculationType,
};

use super::{CoolDown, SkillRecastWindow};

/// 技能是否处于"可施放/就绪"状态，供 UI 显示与施法前置判断共用同一语义。
///
/// 存在未过期的重施窗口时（如锐雯 Q 的多段重施、盲僧 Q 的二段冲刺），
/// 即使主冷却已在计时，技能仍可释放下一段，因此视为就绪；
/// 否则取决于冷却计时器是否结束。
pub fn is_skill_ready(cooldown: &CoolDown, recast: Option<&SkillRecastWindow>) -> bool {
    if let Some(window) = recast {
        if !window.timer.is_finished() {
            return true;
        }
    }
    cooldown.timer.as_ref().map_or(true, |t| t.is_finished())
}

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

/// 读取技能的原始 `dataValues`（按名称），不受 `calculations` 公式约束。
///
/// 与 [`get_skill_value`] 互补：后者只能读 `calculations` 里的计算键，
/// 而很多数值（如冷却退还没收、攻速比例、每秒治疗量）只存在于 `dataValues`。
/// `level` 为 1-based 技能等级；名称匹配忽略大小写与下划线。
pub fn get_skill_data_value(skill_object: &Spell, name: &str, level: usize) -> Option<f32> {
    get_named_data_value(skill_object, name, level)
}

/// 从技能资产的 `castFrame`（30fps 帧数）推算延迟秒数。
///
/// 用于延迟范围伤害（`ActionDelayedDamage`）等需要前摇延迟的技能：
/// 如诺手 Q（7.5）、狗熊 E（25）、瑟提 W（5.265）。
pub fn delay_from_cast_frame(skill_object: &Spell) -> f32 {
    skill_object
        .spell_data
        .as_ref()
        .and_then(|d| d.cast_frame)
        .map(|f| f / 30.0)
        .unwrap_or(0.0)
}

/// 读取技能资产的 `castRadius`（按等级），常作为 AoE 效果半径。
pub fn get_skill_cast_radius(skill_object: &Spell, level: usize) -> Option<f32> {
    let spell_data = skill_object.spell_data.as_ref()?;
    let radii = spell_data.cast_radius.as_ref()?;
    let lvl_idx = if level > 0 { level - 1 } else { 0 };
    Some(*radii.get(lvl_idx).unwrap_or(&0.0))
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
