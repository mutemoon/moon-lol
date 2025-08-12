use crate::league::LeagueLoader;
use bevy::math::Mat4;
use bevy::prelude::*;
use cdragon_prop::{
    BinEmbed, BinEntry, BinFloat, BinHash, BinLink, BinMap, BinMatrix, BinString, BinStruct, BinU32,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LeagueBinCharacterRecord {
    pub character_name: Option<String>,
    pub fallback_character_name: Option<String>,
    pub base_hp: Option<f32>,
    pub base_static_hp_regen: Option<f32>,
    pub health_bar_height: Option<f32>,
    pub base_damage: Option<f32>,
    pub base_armor: Option<f32>,
    pub base_spell_block: Option<f32>,
    pub base_move_speed: Option<f32>,
    pub attack_range: Option<f32>,
    pub attack_speed: Option<f32>,
    pub attack_speed_ratio: Option<f32>,
    pub attack_speed_per_level: Option<f32>,
    pub exp_given_on_death: Option<f32>,
    pub gold_given_on_death: Option<f32>,
    pub local_gold_given_on_death: Option<f32>,
    pub global_gold_given_on_death: Option<f32>,
    pub display_name: Option<String>,
    pub hit_fx_scale: Option<f32>,
    pub selection_height: Option<f32>,
    pub selection_radius: Option<f32>,
    pub pathfinding_collision_radius: Option<f32>,
    pub gameplay_collision_radius: Option<f32>,
    pub unit_tags: Option<String>,
    pub description: Option<String>,
}

impl From<&BinEntry> for LeagueBinCharacterRecord {
    fn from(value: &BinEntry) -> Self {
        let character_name = value
            .getv::<BinString>(LeagueLoader::hash_bin("mCharacterName").into())
            .map(|s| s.0.clone());
        let fallback_character_name = value
            .getv::<BinString>(LeagueLoader::hash_bin("mFallbackCharacterName").into())
            .map(|s| s.0.clone());
        let base_hp = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("baseHP").into())
            .map(|f| f.0);
        let base_static_hp_regen = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("baseStaticHPRegen").into())
            .map(|f| f.0);
        let health_bar_height = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("healthBarHeight").into())
            .map(|f| f.0);
        let base_damage = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("baseDamage").into())
            .map(|f| f.0);
        let base_armor = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("baseArmor").into())
            .map(|f| f.0);
        let base_spell_block = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("baseSpellBlock").into())
            .map(|f| f.0);
        let base_move_speed = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("baseMoveSpeed").into())
            .map(|f| f.0);
        let attack_range = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("attackRange").into())
            .map(|f| f.0);
        let attack_speed = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("attackSpeed").into())
            .map(|f| f.0);
        let attack_speed_ratio = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("attackSpeedRatio").into())
            .map(|f| f.0);
        let attack_speed_per_level = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("attackSpeedPerLevel").into())
            .map(|f| f.0);
        let exp_given_on_death = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("expGivenOnDeath").into())
            .map(|f| f.0);
        let gold_given_on_death = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("goldGivenOnDeath").into())
            .map(|f| f.0);
        let local_gold_given_on_death = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("localGoldGivenOnDeath").into())
            .map(|f| f.0);
        let global_gold_given_on_death = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("globalGoldGivenOnDeath").into())
            .map(|f| f.0);
        let display_name = value
            .getv::<BinString>(LeagueLoader::hash_bin("name").into())
            .map(|s| s.0.clone());
        let hit_fx_scale = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("hitFxScale").into())
            .map(|f| f.0);
        let selection_height = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("selectionHeight").into())
            .map(|f| f.0);
        let selection_radius = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("selectionRadius").into())
            .map(|f| f.0);
        let pathfinding_collision_radius = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("pathfindingCollisionRadius").into())
            .map(|f| f.0);
        let gameplay_collision_radius = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("overrideGameplayCollisionRadius").into())
            .map(|f| f.0);
        let unit_tags = value
            .getv::<BinString>(LeagueLoader::hash_bin("unitTagsString").into())
            .map(|s| s.0.clone());
        let description = value
            .getv::<BinString>(LeagueLoader::hash_bin("description").into())
            .map(|s| s.0.clone());

        LeagueBinCharacterRecord {
            character_name,
            fallback_character_name,
            base_hp,
            base_static_hp_regen,
            health_bar_height,
            base_damage,
            base_armor,
            base_spell_block,
            base_move_speed,
            attack_range,
            attack_speed,
            attack_speed_ratio,
            attack_speed_per_level,
            exp_given_on_death,
            gold_given_on_death,
            local_gold_given_on_death,
            global_gold_given_on_death,
            display_name,
            hit_fx_scale,
            selection_height,
            selection_radius,
            pathfinding_collision_radius,
            gameplay_collision_radius,
            unit_tags,
            description,
        }
    }
}

#[derive(Debug)]
pub struct LeagueBinMaybeCharacterMapRecord {
    pub transform: Mat4,
    pub name: u32,
    pub definition: CharacterMapRecordDefinition,
}

impl From<&BinStruct> for LeagueBinMaybeCharacterMapRecord {
    fn from(value: &BinStruct) -> Self {
        let transform = value
            .getv::<BinMatrix>(LeagueLoader::hash_bin("transform").into())
            .unwrap();

        let name = value
            .getv::<BinHash>(LeagueLoader::hash_bin("name").into())
            .unwrap();

        let definition = value
            .getv::<BinStruct>(LeagueLoader::hash_bin("definition").into())
            .unwrap();

        Self {
            transform: Mat4::from_cols_array_2d(&transform.0),
            name: name.0.hash,
            definition: definition.into(),
        }
    }
}

#[derive(Debug)]
pub struct CharacterMapRecordDefinition {
    pub team: Option<u32>,
    pub character_record: String,
    pub skin: String,
}

impl From<&BinStruct> for CharacterMapRecordDefinition {
    fn from(value: &BinStruct) -> Self {
        let team = value
            .getv::<BinU32>(LeagueLoader::hash_bin("Team").into())
            .map(|v| v.0);

        let character_record = value
            .getv::<BinString>(LeagueLoader::hash_bin("CharacterRecord").into())
            .unwrap();

        let skin = value
            .getv::<BinString>(LeagueLoader::hash_bin("Skin").into())
            .unwrap();

        Self {
            team,
            character_record: character_record.0.clone(),
            skin: skin.0.clone(),
        }
    }
}

pub struct SkinCharacterDataProperties {
    pub skin_animation_properties: SkinAnimationProperties,
    pub skin_mesh_properties: SkinMeshDataProperties,
}

impl From<&BinEntry> for SkinCharacterDataProperties {
    fn from(value: &BinEntry) -> Self {
        let skin_animation_properties = value
            .getv::<BinEmbed>(LeagueLoader::hash_bin("skinAnimationProperties").into())
            .unwrap();

        let skin_mesh_properties = value
            .getv::<BinEmbed>(LeagueLoader::hash_bin("skinMeshProperties").into())
            .unwrap();

        Self {
            skin_animation_properties: skin_animation_properties.into(),
            skin_mesh_properties: skin_mesh_properties.into(),
        }
    }
}

pub struct SkinMeshDataProperties {
    pub skeleton: String,
    pub simple_skin: String,
    pub texture: String,
    pub skin_scale: Option<f32>,
}

impl From<&BinEmbed> for SkinMeshDataProperties {
    fn from(value: &BinEmbed) -> Self {
        let skeleton = value
            .getv::<BinString>(LeagueLoader::hash_bin("skeleton").into())
            .map(|v| v.0.clone())
            .unwrap();

        let simple_skin = value
            .getv::<BinString>(LeagueLoader::hash_bin("simpleSkin").into())
            .unwrap();

        let texture = value
            .getv::<BinString>(LeagueLoader::hash_bin("texture").into())
            .unwrap();

        let skin_scale = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("skinScale").into())
            .map(|v| v.0);

        Self {
            skeleton,
            simple_skin: simple_skin.0.clone(),
            texture: texture.0.clone(),
            skin_scale,
        }
    }
}

pub struct SkinAnimationProperties {
    pub animation_graph_data: u32,
}

impl From<&BinEmbed> for SkinAnimationProperties {
    fn from(value: &BinEmbed) -> Self {
        let animation_graph_data = value
            .getv::<BinLink>(LeagueLoader::hash_bin("animationGraphData").into())
            .map(|v| v.0.hash)
            .unwrap();

        Self {
            animation_graph_data,
        }
    }
}

#[derive(Debug)]
pub struct AnimationGraphData {
    pub clip_data_map: HashMap<u32, AnimationClipData>,
}

impl From<&BinEntry> for AnimationGraphData {
    fn from(value: &BinEntry) -> Self {
        let clip_data_map = value
            .getv::<BinMap>(LeagueLoader::hash_bin("mClipDataMap").into())
            .unwrap()
            .downcast::<BinHash, BinStruct>()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.0.hash, v.into()))
            .collect();

        Self { clip_data_map }
    }
}

#[derive(Debug)]
pub enum AnimationClipData {
    AtomicClipData {
        animation_resource_data: AnimationResourceData,
    },
    Unknown,
}

impl From<&BinStruct> for AnimationClipData {
    fn from(value: &BinStruct) -> Self {
        let hash = LeagueLoader::hash_bin("AtomicClipData");

        if value.ctype.hash == hash {
            AnimationClipData::AtomicClipData {
                animation_resource_data: value
                    .getv::<BinEmbed>(LeagueLoader::hash_bin("mAnimationResourceData").into())
                    .unwrap()
                    .into(),
            }
        } else {
            AnimationClipData::Unknown
        }
    }
}

#[derive(Debug)]
pub struct AnimationResourceData {
    pub animation_file_path: String,
}

impl From<&BinEmbed> for AnimationResourceData {
    fn from(value: &BinEmbed) -> Self {
        let animation_file_path = value
            .getv::<BinString>(LeagueLoader::hash_bin("mAnimationFilePath").into())
            .map(|v| v.0.clone())
            .unwrap();

        AnimationResourceData {
            animation_file_path,
        }
    }
}
