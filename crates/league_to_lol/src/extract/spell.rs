use std::collections::BTreeMap;

use league_core::extract::{
    EnumGameCalculation, GameCalculation, SpellDataResource, SpellDataValue, SpellEffectAmount,
    SpellObject,
};
use league_loader::game::{Data, PropGroup};
use lol_base::movement::{MissileSpecification, MovementType, MovementTypeFixedSpeed};
use lol_base::spell::{DataSpell, Spell, ValuesData, ValuesEffect};
use lol_base::spell_calc::{
    CalculationPart, CalculationPartEffectValue, CalculationPartNamedDataValue,
    CalculationPartStatCoefficient, CalculationPartStatNamedDataValue, CalculationPartStatSub,
    CalculationSpell, CalculationType,
};

use super::utils::write_to_file;

/// 从 CharacterRecord 所在 bin 文件提取所有 SpellObject，转换为 DataSpell 并导出
pub fn extract_spells_for_champion(champ_name: &str, prop_group: &PropGroup) {
    let spells = prop_group.get_all_by_class::<SpellObject>();

    if spells.is_empty() {
        println!("[WARN] 未在 {} bin 中找到 SpellObject", champ_name);
        return;
    }

    println!(
        "[INFO] 为 {} 找到 {} 个 SpellObject",
        champ_name,
        spells.len()
    );

    for spell_obj in spells {
        let object_name = &spell_obj.object_name;
        if object_name.is_empty() {
            continue;
        }

        let Some(spell_data) = &spell_obj.m_spell else {
            continue;
        };

        let data_spell = convert_spell_data_resource(spell_data);

        let spell = Spell {
            spell_data: Some(data_spell),
        };

        let output_dir = format!("assets/characters/{}/spells", champ_name.to_lowercase());
        let output_path = format!("{}/{}.ron", output_dir, object_name);

        // 确保目录存在
        if let Err(e) = std::fs::create_dir_all(&output_dir) {
            println!("[WARN] 无法创建目录 {}: {}", output_dir, e);
            continue;
        }

        let serialized = ron::ser::to_string_pretty(&spell, ron::ser::PrettyConfig::default())
            .map_err(|e| format!("序列化失败: {}", e));

        match serialized {
            Ok(s) => {
                write_to_file(&output_path, s);
                println!("[INFO] 已导出技能: {}", output_path);
            }
            Err(e) => {
                println!("[WARN] 序列化 {} 失败: {}", object_name, e);
            }
        }
    }
}

/// 将 SpellDataResource 转换为 DataSpell
fn convert_spell_data_resource(spell: &SpellDataResource) -> DataSpell {
    DataSpell {
        calculations: spell
            .m_spell_calculations
            .as_ref()
            .map(convert_calculations),
        effect_amounts: convert_effect_amounts(&spell.m_effect_amount),
        data_values: convert_data_values(&spell.data_values),
        mana: spell.mana.clone(),
        missile_spec: convert_missile_spec(&spell.m_missile_spec),
        hit_bone_name: spell.m_hit_bone_name.clone(),
        missile_speed: spell.missile_speed,
        missile_effect_key: spell.m_missile_effect_key,
        cast_type: spell.m_cast_type,
    }
}

/// 转换技能效果值
fn convert_effect_amounts(effects: &Option<Vec<SpellEffectAmount>>) -> Option<Vec<ValuesEffect>> {
    effects.as_ref().map(|effects| {
        effects
            .iter()
            .map(|e| ValuesEffect {
                values: e.value.clone(),
            })
            .collect()
    })
}

/// 转换技能数据值
fn convert_data_values(values: &Option<Vec<SpellDataValue>>) -> Option<Vec<ValuesData>> {
    values.as_ref().map(|values| {
        values
            .iter()
            .map(|v| ValuesData {
                name: v.name.clone(),
                values: v.values.clone(),
            })
            .collect()
    })
}

/// 转换导弹规格
fn convert_missile_spec(
    spec: &Option<league_core::extract::MissileSpecification>,
) -> Option<MissileSpecification> {
    spec.as_ref().map(|spec| {
        let movement = convert_movement(&spec.movement_component);
        MissileSpecification {
            movement_component: movement,
        }
    })
}

/// 转换移动类型
fn convert_movement(movement: &league_core::extract::EnumMovement) -> MovementType {
    match movement {
        league_core::extract::EnumMovement::FixedSpeedMovement(fixed) => {
            MovementType::MovementTypeFixedSpeed(MovementTypeFixedSpeed {
                speed: fixed.m_speed,
                start_bone_name: fixed.m_start_bone_name.clone(),
            })
        }
        _ => {
            // 其他移动类型暂时使用默认构造
            MovementType::MovementTypeFixedSpeed(MovementTypeFixedSpeed {
                speed: None,
                start_bone_name: None,
            })
        }
    }
}

/// 转换计算公式
fn convert_calculations(
    calcs: &BTreeMap<u32, EnumGameCalculation>,
) -> BTreeMap<u32, CalculationType> {
    calcs
        .iter()
        .filter_map(|(key, calc)| {
            let converted = convert_enum_calculation(calc);
            converted.map(|c| (*key, c))
        })
        .collect()
}

/// 转换计算枚举
fn convert_enum_calculation(calc: &EnumGameCalculation) -> Option<CalculationType> {
    match calc {
        EnumGameCalculation::GameCalculation(gc) => Some(CalculationType::CalculationSpell(
            convert_game_calculation(gc),
        )),
        _ => {
            // 暂不支持 Conditional 和 Modified
            None
        }
    }
}

/// 转换游戏计算
fn convert_game_calculation(gc: &GameCalculation) -> CalculationSpell {
    CalculationSpell {
        formula_parts: gc.m_formula_parts.as_ref().map(|parts| {
            parts
                .iter()
                .filter_map(|p| convert_calculation_part(p))
                .collect()
        }),
        multiplier: gc
            .m_multiplier
            .as_ref()
            .and_then(|m| convert_calculation_part(m)),
        precision: gc.m_precision,
    }
}

/// 转换计算部件
fn convert_calculation_part(
    part: &league_core::extract::EnumAbilityResourceByCoefficientCalculationPart,
) -> Option<CalculationPart> {
    match part {
        league_core::extract::EnumAbilityResourceByCoefficientCalculationPart::EffectValueCalculationPart(
            p,
        ) => Some(CalculationPart::CalculationPartEffectValue(
            CalculationPartEffectValue {
                effect_index: p.m_effect_index,
            },
        )),
        league_core::extract::EnumAbilityResourceByCoefficientCalculationPart::StatByCoefficientCalculationPart(
            p,
        ) => Some(CalculationPart::CalculationPartStatCoefficient(
            CalculationPartStatCoefficient {
                stat: p.m_stat,
                coefficient: p.m_coefficient,
                stat_formula: p.m_stat_formula,
            },
        )),
        league_core::extract::EnumAbilityResourceByCoefficientCalculationPart::NamedDataValueCalculationPart(
            p,
        ) => Some(CalculationPart::CalculationPartNamedDataValue(
            CalculationPartNamedDataValue {
                data_value: p.m_data_value,
            },
        )),
        league_core::extract::EnumAbilityResourceByCoefficientCalculationPart::StatBySubPartCalculationPart(
            p,
        ) => {
            let subpart = convert_calculation_part(&p.m_subpart);
            Some(CalculationPart::CalculationPartStatSub(
                CalculationPartStatSub {
                    stat: p.m_stat,
                    subpart: subpart.map(Box::new),
                },
            ))
        }
        league_core::extract::EnumAbilityResourceByCoefficientCalculationPart::StatByNamedDataValueCalculationPart(
            p,
        ) => Some(CalculationPart::CalculationPartStatNamedDataValue(
            CalculationPartStatNamedDataValue {
                stat: p.m_stat,
                data_value: p.m_data_value,
            },
        )),
        _ => None,
    }
}
