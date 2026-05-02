use serde::{Deserialize, Serialize};

/// 计算部件类型
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum CalculationPart {
    CalculationPartEffectValue(CalculationPartEffectValue),
    CalculationPartStatCoefficient(CalculationPartStatCoefficient),
    CalculationPartNamedDataValue(CalculationPartNamedDataValue),
    CalculationPartStatSub(CalculationPartStatSub),
    CalculationPartStatNamedDataValue(CalculationPartStatNamedDataValue),
    // ... 其他变体暂不实现
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalculationPartEffectValue {
    pub effect_index: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalculationPartStatCoefficient {
    pub stat: Option<u8>,
    pub coefficient: Option<f32>,
    pub stat_formula: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalculationPartNamedDataValue {
    pub data_value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalculationPartStatSub {
    pub stat: Option<u8>,
    pub subpart: Option<Box<CalculationPart>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalculationPartStatNamedDataValue {
    pub stat: Option<u8>,
    pub data_value: String,
}

/// 计算类型
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum CalculationType {
    CalculationSpell(CalculationSpell),
    // Conditional 和 Modified 暂不实现
}

/// 技能计算容器
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalculationSpell {
    pub formula_parts: Option<Vec<CalculationPart>>,
    pub multiplier: Option<CalculationPart>,
    pub precision: Option<i32>,
}
