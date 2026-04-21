use bevy::asset::Asset;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

/// 角色数据记录
#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct CharacterRecord {
    pub primary_ability_resource: Option<AbilityResourceData>,
    pub pathfinding_collision_radius: Option<f32>,
    pub health_bar_height: Option<f32>,
    pub exp_given_on_death: Option<f32>,
    pub experience_radius: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbilityResourceData {
    pub ar_type: u8,
}
