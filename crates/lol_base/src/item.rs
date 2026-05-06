use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath, Default)]
pub struct ConfigItem {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub price: i32,
    pub icon_path: String,
}
