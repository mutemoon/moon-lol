use std::collections::BTreeMap;

use bevy::asset::Asset;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

use super::movement::MissileSpecification;
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
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataSpell {
    pub calculations: Option<BTreeMap<String, CalculationType>>,
    pub effect_amounts: Option<Vec<ValuesEffect>>,
    pub data_values: Option<Vec<ValuesData>>,
    pub mana: Option<Vec<f32>>,
    pub missile_spec: Option<MissileSpecification>,
    pub hit_bone_name: Option<String>,
    pub missile_speed: Option<f32>,
    pub missile_effect_key: Option<u32>,
    pub cast_type: Option<u32>,
    pub cast_range: Option<Vec<f32>>,
    pub cast_radius: Option<Vec<f32>>,
    pub cast_cone_angle: Option<f32>,
    pub cast_cone_distance: Option<f32>,
    pub line_width: Option<f32>,
    pub cast_frame: Option<f32>,
    pub animation_name: Option<String>,
    pub cooldown_time: Option<Vec<f32>>,
    pub cant_cancel_while_winding_up: Option<bool>,
    pub spell_reveals_champion: Option<bool>,
    pub affects_type_flags: Option<u32>,
    pub alternate_name: Option<String>,
    pub coefficient: Option<f32>,
    pub hit_effect_key: Option<u32>,
    pub selection_priority: Option<u32>,
    pub use_animator_framerate: Option<bool>,
}

/// 技能对象 (Bevy Asset)
#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct Spell {
    pub spell_data: Option<DataSpell>,
}
