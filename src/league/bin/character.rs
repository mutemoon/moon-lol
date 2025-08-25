use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::league::BinLink;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterRecord {
    #[serde(default)]
    pub attack_range: f32,
    #[serde(default)]
    pub attack_speed_per_level: f32,
    #[serde(default)]
    pub attack_speed_ratio: f32,
    #[serde(default)]
    pub attack_speed: f32,
    pub base_armor: Option<f32>,
    #[serde(default)]
    pub base_damage: f32,
    pub base_hp: f32,
    pub base_move_speed: f32,
    pub base_spell_block: Option<f32>,
    pub base_static_hp_regen: f32,
    pub character_name: Option<String>,
    pub description: Option<String>,
    pub display_name: Option<String>,
    pub exp_given_on_death: f32,
    pub fallback_character_name: Option<String>,
    #[serde(default)]
    pub gameplay_collision_radius: f32,
    #[serde(default)]
    pub global_gold_given_on_death: f32,
    #[serde(default)]
    pub gold_given_on_death: f32,
    pub health_bar_height: f32,
    #[serde(default)]
    pub hit_fx_scale: f32,
    #[serde(default)]
    pub local_gold_given_on_death: f32,
    pub pathfinding_collision_radius: f32,
    pub selection_height: f32,
    pub selection_radius: f32,
    pub unit_tags: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinCharacterDataProperties {
    pub skin_animation_properties: SkinAnimationProperties,
    pub skin_mesh_properties: SkinMeshDataProperties,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinMeshDataProperties {
    pub skeleton: String,
    pub simple_skin: String,
    pub texture: String,
    pub skin_scale: Option<f32>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinAnimationProperties {
    pub animation_graph_data: BinLink,
}
