use std::collections::{BTreeMap, HashMap};

use league_core::extract::{
    EnumGameCalculation, GameCalculation, SpellDataResource, SpellDataValue, SpellEffectAmount,
    SpellObject,
};
use league_loader::game::{Data, PropGroup};
use league_utils::hash_to_field_name;
use lol_base::movement::{
    HeightSolver, MissileBehavior, MissileSpecification, MovementType, MovementTypeFixedSpeed,
    VerticalFacing,
};
use lol_base::spell::{DataSpell, Spell, ValuesData, ValuesEffect};
use lol_base::spell_calc::{
    CalculationPart, CalculationPartEffectValue, CalculationPartNamedDataValue,
    CalculationPartStatCoefficient, CalculationPartStatNamedDataValue, CalculationPartStatSub,
    CalculationSpell, CalculationType,
};

use super::utils::write_to_file;

/// 从 CharacterRecord 所在 bin 文件提取所有 SpellObject，转换为 DataSpell 并导出
/// 返回所有技能对象名称列表
pub fn extract_spells_for_champion(
    champ_name: &str,
    prop_group: &PropGroup,
    hashes: &HashMap<u32, String>,
) -> Vec<String> {
    let spells = prop_group.get_all_by_class::<SpellObject>();

    if spells.is_empty() {
        println!("[WARN] 未在 {} bin 中找到 SpellObject", champ_name);
        return Vec::new();
    }

    println!(
        "[INFO] 为 {} 找到 {} 个 SpellObject",
        champ_name,
        spells.len()
    );

    let mut spell_names = Vec::new();

    for spell_obj in spells {
        let object_name = &spell_obj.object_name;
        if object_name.is_empty() {
            continue;
        }

        let Some(spell_data) = &spell_obj.m_spell else {
            continue;
        };

        let data_spell = convert_spell_data_resource(spell_data, hashes);

        let spell = Spell {
            spell_data: Some(data_spell),
        };

        let output_dir = format!("assets/characters/{}/spells", champ_name);
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
                // println!("[INFO] 已导出技能: {}", output_path);
                spell_names.push(object_name.clone());
            }
            Err(e) => {
                println!("[WARN] 序列化 {} 失败: {}", object_name, e);
            }
        }
    }

    spell_names
}

/// 将 SpellDataResource 转换为 DataSpell
fn convert_spell_data_resource(
    spell: &SpellDataResource,
    hashes: &HashMap<u32, String>,
) -> DataSpell {
    DataSpell {
        calculations: spell
            .m_spell_calculations
            .as_ref()
            .map(|calcs| convert_calculations(calcs, hashes)),
        effect_amounts: convert_effect_amounts(&spell.m_effect_amount),
        data_values: convert_data_values(&spell.data_values),
        mana: spell.mana.clone(),
        missile_spec: convert_missile_spec(&spell.m_missile_spec),
        hit_bone_name: spell.m_hit_bone_name.clone(),
        missile_speed: spell.missile_speed,
        missile_effect_key: spell.m_missile_effect_key,
        cast_type: spell.m_cast_type,
        cast_range: spell.cast_range.clone(),
        cast_radius: spell.cast_radius.clone(),
        cast_cone_angle: spell.cast_cone_angle,
        cast_cone_distance: spell.cast_cone_distance,
        line_width: spell.m_line_width,
        cast_frame: spell.cast_frame,
        animation_name: spell.m_animation_name.clone(),
        cooldown_time: spell.cooldown_time.clone(),
        cant_cancel_while_winding_up: spell.m_cant_cancel_while_winding_up,
        spell_reveals_champion: spell.m_spell_reveals_champion,
        affects_type_flags: spell.m_affects_type_flags,
        alternate_name: spell.m_alternate_name.clone(),
        coefficient: spell.m_coefficient,
        hit_effect_key: spell.m_hit_effect_key,
        selection_priority: spell.selection_priority,
        use_animator_framerate: spell.use_animator_framerate,
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
    use league_core::extract::{EnumCastOnHit, EnumFacing, EnumHeightSolver};

    spec.as_ref().map(|spec| {
        let movement = convert_movement(&spec.movement_component);
        let behaviors = spec.behaviors.as_ref().map(|behaviors| {
            behaviors
                .iter()
                .filter_map(|b| match b {
                    EnumCastOnHit::CastOnHit => Some(MissileBehavior::CastOnHit),
                    EnumCastOnHit::DestroyOnMovementComplete(_) => {
                        Some(MissileBehavior::DestroyOnMovementComplete)
                    }
                    _ => None,
                })
                .collect()
        });
        let height_solver = spec.height_solver.as_ref().and_then(|h| match h {
            EnumHeightSolver::BlendedLinearHeightSolver => {
                Some(HeightSolver::BlendedLinearHeightSolver)
            }
            _ => None,
        });
        let vertical_facing = spec.vertical_facing.as_ref().and_then(|f| match f {
            EnumFacing::VerticalFacingFaceTarget => Some(VerticalFacing::VerticalFacingFaceTarget),
            _ => None,
        });
        MissileSpecification {
            movement_component: movement,
            missile_width: spec.m_missile_width,
            behaviors,
            height_solver,
            vertical_facing,
        }
    })
}

/// 转换移动类型
fn convert_movement(movement: &Option<league_core::extract::EnumMovement>) -> MovementType {
    match movement {
        Some(league_core::extract::EnumMovement::FixedSpeedMovement(fixed)) => {
            MovementType::MovementTypeFixedSpeed(MovementTypeFixedSpeed {
                speed: fixed.m_speed,
                start_bone_name: fixed.m_start_bone_name.clone(),
                tracks_target: fixed.m_tracks_target,
                project_target_to_cast_range: fixed.m_project_target_to_cast_range,
                use_height_offset_at_end: fixed.m_use_height_offset_at_end,
                offset_initial_target_height: fixed.m_offset_initial_target_height,
            })
        }
        _ => MovementType::MovementTypeFixedSpeed(MovementTypeFixedSpeed {
            speed: None,
            start_bone_name: None,
            tracks_target: None,
            project_target_to_cast_range: None,
            use_height_offset_at_end: None,
            offset_initial_target_height: None,
        }),
    }
}

/// 转换计算公式
fn convert_calculations(
    calcs: &BTreeMap<u32, EnumGameCalculation>,
    hashes: &HashMap<u32, String>,
) -> BTreeMap<String, CalculationType> {
    calcs
        .iter()
        .filter_map(|(key, calc)| {
            let converted = convert_enum_calculation(calc, hashes);
            converted.map(|c| (hash_to_field_name(key, hashes), c))
        })
        .collect()
}

/// 转换计算枚举
fn convert_enum_calculation(
    calc: &EnumGameCalculation,
    hashes: &HashMap<u32, String>,
) -> Option<CalculationType> {
    match calc {
        EnumGameCalculation::GameCalculation(gc) => Some(CalculationType::CalculationSpell(
            convert_game_calculation(gc, hashes),
        )),
        _ => {
            // 暂不支持 Conditional 和 Modified
            None
        }
    }
}

/// 转换游戏计算
fn convert_game_calculation(
    gc: &GameCalculation,
    hashes: &HashMap<u32, String>,
) -> CalculationSpell {
    CalculationSpell {
        formula_parts: gc.m_formula_parts.as_ref().map(|parts| {
            parts
                .iter()
                .filter_map(|p| convert_calculation_part(p, hashes))
                .collect()
        }),
        multiplier: gc
            .m_multiplier
            .as_ref()
            .and_then(|m| convert_calculation_part(m, hashes)),
        precision: gc.m_precision,
    }
}

/// 转换计算部件
fn convert_calculation_part(
    part: &league_core::extract::EnumAbilityResourceByCoefficientCalculationPart,
    hashes: &HashMap<u32, String>,
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
                data_value: hash_to_field_name(&p.m_data_value, hashes),
            },
        )),
        league_core::extract::EnumAbilityResourceByCoefficientCalculationPart::StatBySubPartCalculationPart(
            p,
        ) => {
            let subpart = convert_calculation_part(&p.m_subpart, hashes);
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
                data_value: hash_to_field_name(&p.m_data_value, hashes),
            },
        )),
        _ => None,
    }
}
