use bevy::prelude::*;

#[derive(Component)]
pub struct AbilityResource {
    pub ar_type: AbilityResourceType,
    pub value: f32,
    pub max: f32,
    pub base: f32,
    pub per_level: f32,
    pub base_static_regen: f32,
    pub regen_per_level: f32,
}

pub enum AbilityResourceType {
    Mana,
    Energy,
    Turret,
    Camp,
}

impl From<u8> for AbilityResourceType {
    fn from(value: u8) -> Self {
        match value {
            0 => AbilityResourceType::Mana,
            1 => AbilityResourceType::Energy,
            7 => AbilityResourceType::Turret,
            8 => AbilityResourceType::Camp,
            _ => panic!("Invalid AbilityResourceType {}", value),
        }
    }
}

impl From<Option<u8>> for AbilityResourceType {
    fn from(value: Option<u8>) -> Self {
        match value {
            Some(value) => AbilityResourceType::from(value),
            None => AbilityResourceType::Mana,
        }
    }
}
