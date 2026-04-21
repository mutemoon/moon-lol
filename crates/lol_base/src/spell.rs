use std::collections::HashMap;

use bevy::asset::Asset;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

use super::spell_calc::CalculationType;

/// 技能效果值
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValuesEffect {
    pub values: Option<Vec<f32>>,
}

/// 技能数据值
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValuesData {
    pub name: String,
    pub values: Option<Vec<f32>>,
}

/// 技能数据资源
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataSpell {
    pub calculations: Option<HashMap<u32, CalculationType>>,
    pub effect_amounts: Option<Vec<ValuesEffect>>,
    pub data_values: Option<Vec<ValuesData>>,
    pub mana: Option<Vec<f32>>,
}

/// 技能对象 (Bevy Asset)
#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct Spell {
    pub spell_data: Option<DataSpell>,
}
