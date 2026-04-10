use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect, Serialize, Deserialize, Debug)]
#[reflect(Component)]
pub struct ConfigCharacter {
    pub skin_path: String,
    pub character_record: String,
}
