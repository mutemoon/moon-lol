use bevy::prelude::*;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ConfigCharacter {
    pub skin_path: String,
    pub character_record: Handle<DynamicWorld>,
}
